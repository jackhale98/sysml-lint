/// Extract action flow models from tree-sitter parse trees.

use tree_sitter::{Node, Parser};

use crate::model::Span;
use crate::parser::{get_language, node_text};
use crate::sim::action_flow::*;
use crate::sim::expr_parser::extract_expr;

/// Extract all action definitions from source.
pub fn extract_actions(file: &str, source: &str) -> Vec<ActionModel> {
    let mut parser = Parser::new();
    parser.set_language(&get_language()).unwrap();
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let source_bytes = source.as_bytes();
    let mut results = Vec::new();
    collect_action_defs(tree.root_node(), source_bytes, file, &mut results);
    results
}

fn collect_action_defs(
    node: Node,
    source: &[u8],
    _file: &str,
    results: &mut Vec<ActionModel>,
) {
    if node.kind() == "action_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = node_text(&name_node, source).to_string();
            let mut steps = Vec::new();

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "definition_body" {
                    extract_action_body(&child, source, &mut steps);
                }
            }

            results.push(ActionModel {
                name,
                steps,
                span: Span::from_node(&node),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_action_defs(child, source, _file, results);
    }
}

fn extract_action_body(body: &Node, source: &[u8], steps: &mut Vec<ActionStep>) {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if let Some(step) = extract_step(&child, source) {
            steps.push(step);
        }
    }
}

fn extract_step(node: &Node, source: &[u8]) -> Option<ActionStep> {
    match node.kind() {
        "action_usage" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| node_text(&n, source).to_string())?;
            Some(ActionStep::Perform {
                name,
                span: Span::from_node(node),
            })
        }
        "perform_statement" => {
            let mut name = None;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if matches!(
                    child.kind(),
                    "identifier" | "qualified_name" | "feature_chain"
                ) {
                    let text = node_text(&child, source).to_string();
                    if text != "perform" && text != "action" {
                        name = Some(text);
                        break;
                    }
                }
            }
            name.map(|n| ActionStep::Perform {
                name: n,
                span: Span::from_node(node),
            })
        }
        "then_succession" => {
            // `then actionName;` — a sequential step
            let mut name = None;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if matches!(
                    child.kind(),
                    "identifier" | "qualified_name" | "feature_chain"
                ) {
                    let text = node_text(&child, source).to_string();
                    if text != "then" && text != "action" {
                        name = Some(text);
                        break;
                    }
                }
                // Check for nested action_usage inside then_succession
                if child.kind() == "action_usage" {
                    if let Some(step) = extract_step(&child, source) {
                        return Some(step);
                    }
                }
            }
            name.map(|n| ActionStep::Perform {
                name: n,
                span: Span::from_node(node),
            })
        }
        "succession_statement" => {
            // `first A then B;`
            let mut refs = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if matches!(
                    child.kind(),
                    "identifier" | "qualified_name" | "feature_chain"
                ) {
                    let text = node_text(&child, source).to_string();
                    if text != "first" && text != "then" {
                        refs.push(text);
                    }
                }
            }
            if refs.len() >= 2 {
                Some(ActionStep::Sequence {
                    steps: refs
                        .into_iter()
                        .map(|name| ActionStep::Perform {
                            name,
                            span: Span::from_node(node),
                        })
                        .collect(),
                    span: Span::from_node(node),
                })
            } else {
                None
            }
        }
        "fork_node" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| node_text(&n, source).to_string());
            let mut branches = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "definition_body" {
                    extract_action_body(&child, source, &mut branches);
                }
            }
            Some(ActionStep::Fork {
                name,
                branches,
                span: Span::from_node(node),
            })
        }
        "join_node" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| node_text(&n, source).to_string());
            Some(ActionStep::Join {
                name,
                span: Span::from_node(node),
            })
        }
        "decide_node" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| node_text(&n, source).to_string());
            // Decision branches would be extracted from body
            Some(ActionStep::Decide {
                name,
                branches: Vec::new(),
                span: Span::from_node(node),
            })
        }
        "merge_node" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| node_text(&n, source).to_string());
            Some(ActionStep::Merge {
                name,
                span: Span::from_node(node),
            })
        }
        "if_action" => extract_if_action(node, source),
        "assign_action" => extract_assign_action(node, source),
        "send_action" => extract_send_action(node, source),
        "while_action" => extract_while_action(node, source),
        "for_action" => extract_for_action(node, source),
        _ => None,
    }
}

fn extract_if_action(node: &Node, source: &[u8]) -> Option<ActionStep> {
    let mut condition = None;
    let mut then_ref = None;
    let mut else_ref = None;
    let mut saw_then = false;
    let mut saw_else = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "then" => saw_then = true,
            "else" => saw_else = true,
            "if_action" => {
                // Nested if-else chain
                if saw_else {
                    else_ref = extract_if_action(&child, source);
                }
            }
            "identifier" | "qualified_name" | "feature_chain" => {
                let text = node_text(&child, source).to_string();
                if text == "if" || text == "then" || text == "else" {
                    continue;
                }
                if saw_else {
                    else_ref = Some(ActionStep::Perform {
                        name: text,
                        span: Span::from_node(&child),
                    });
                } else if saw_then {
                    then_ref = Some(ActionStep::Perform {
                        name: text,
                        span: Span::from_node(&child),
                    });
                } else if condition.is_none() {
                    // Try to extract as expression
                    condition = extract_expr(&child, source).ok();
                }
            }
            _ => {
                if condition.is_none() && child.is_named() && !saw_then {
                    condition = extract_expr(&child, source).ok();
                }
            }
        }
    }

    let cond = condition?;
    let then_step = then_ref?;

    Some(ActionStep::IfAction {
        condition: cond,
        then_step: Box::new(then_step),
        else_step: else_ref.map(Box::new),
        span: Span::from_node(node),
    })
}

fn extract_assign_action(node: &Node, source: &[u8]) -> Option<ActionStep> {
    let mut target = None;
    let mut value = None;
    let mut saw_assign_op = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "qualified_name" | "feature_chain" => {
                let text = node_text(&child, source).to_string();
                if text == "assign" {
                    continue;
                }
                if saw_assign_op {
                    value = extract_expr(&child, source).ok();
                } else {
                    target = Some(text);
                }
            }
            _ => {
                let text = node_text(&child, source).trim().to_string();
                if text == ":=" {
                    saw_assign_op = true;
                } else if saw_assign_op && child.is_named() && value.is_none() {
                    value = extract_expr(&child, source).ok();
                }
            }
        }
    }

    let tgt = target?;
    let val = value?;

    Some(ActionStep::Assign {
        target: tgt,
        value: val,
        span: Span::from_node(node),
    })
}

fn extract_send_action(node: &Node, source: &[u8]) -> Option<ActionStep> {
    let mut payload = None;
    let mut via = None;
    let mut to = None;
    let mut after_via = false;
    let mut after_to = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "via" => after_via = true,
            "to" => after_to = true,
            "identifier" | "qualified_name" | "feature_chain" => {
                let text = node_text(&child, source).to_string();
                if text == "send" {
                    continue;
                }
                if after_to {
                    to = Some(text);
                    after_to = false;
                } else if after_via {
                    via = Some(text);
                    after_via = false;
                } else if payload.is_none() {
                    payload = Some(text);
                }
            }
            _ => {}
        }
    }

    Some(ActionStep::Send {
        payload,
        via,
        to,
        span: Span::from_node(node),
    })
}

fn extract_while_action(node: &Node, source: &[u8]) -> Option<ActionStep> {
    let mut condition = None;
    let mut body_ref = None;
    let mut saw_do = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "do" => saw_do = true,
            "identifier" | "qualified_name" | "feature_chain" => {
                let text = node_text(&child, source).to_string();
                if text == "while" || text == "do" {
                    continue;
                }
                if saw_do {
                    body_ref = Some(ActionStep::Perform {
                        name: text,
                        span: Span::from_node(&child),
                    });
                } else if condition.is_none() {
                    condition = extract_expr(&child, source).ok();
                }
            }
            _ => {
                if condition.is_none() && child.is_named() && !saw_do {
                    condition = extract_expr(&child, source).ok();
                }
            }
        }
    }

    let cond = condition?;
    let body = body_ref?;

    Some(ActionStep::WhileLoop {
        condition: cond,
        body: Box::new(body),
        span: Span::from_node(node),
    })
}

fn extract_for_action(node: &Node, source: &[u8]) -> Option<ActionStep> {
    let mut variable = None;
    let mut collection = None;
    let mut body_ref = None;
    let mut saw_in = false;
    let mut saw_do = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "in" => saw_in = true,
            "do" => saw_do = true,
            "identifier" | "qualified_name" | "feature_chain" => {
                let text = node_text(&child, source).to_string();
                if text == "for" || text == "in" || text == "do" {
                    continue;
                }
                if saw_do {
                    body_ref = Some(ActionStep::Perform {
                        name: text,
                        span: Span::from_node(&child),
                    });
                } else if saw_in {
                    collection = Some(text);
                } else if variable.is_none() {
                    variable = Some(text);
                }
            }
            _ => {}
        }
    }

    let var = variable?;
    let coll = collection?;
    let body = body_ref.unwrap_or(ActionStep::Done {
        span: Span::from_node(node),
    });

    Some(ActionStep::ForLoop {
        variable: var,
        collection: coll,
        body: Box::new(body),
        span: Span::from_node(node),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_action() {
        let source = r#"
            action def ProcessOrder {
                action validate;
                then action ship;
                then action notify;
            }
        "#;
        let actions = extract_actions("test.sysml", source);
        assert_eq!(actions.len(), 1);
        let a = &actions[0];
        assert_eq!(a.name, "ProcessOrder");
        assert!(a.steps.len() >= 1, "expected steps, got {}", a.steps.len());
    }

    #[test]
    fn extract_action_with_succession() {
        let source = r#"
            action def Pipeline {
                action step1;
                action step2;
                action step3;
                first step1 then step2;
                first step2 then step3;
            }
        "#;
        let actions = extract_actions("test.sysml", source);
        assert_eq!(actions.len(), 1);
        let a = &actions[0];
        // Should have action usages + succession statements
        assert!(a.steps.len() >= 3);
    }

    #[test]
    fn no_actions_in_part_file() {
        let source = "part def Vehicle;";
        let actions = extract_actions("test.sysml", source);
        assert!(actions.is_empty());
    }
}
