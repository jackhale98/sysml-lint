/// Extract state machine models from tree-sitter parse trees.

use tree_sitter::{Node, Parser};

use crate::model::Span;
use crate::parser::{get_language, node_text};
use crate::sim::expr_parser::extract_expr;
use crate::sim::state_machine::*;

/// Extract all state machine definitions from source.
pub fn extract_state_machines(file: &str, source: &str) -> Vec<StateMachineModel> {
    let mut parser = Parser::new();
    parser.set_language(&get_language()).unwrap();
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let source_bytes = source.as_bytes();
    let mut results = Vec::new();
    collect_state_defs(tree.root_node(), source_bytes, file, &mut results);
    results
}

fn collect_state_defs(
    node: Node,
    source: &[u8],
    _file: &str,
    results: &mut Vec<StateMachineModel>,
) {
    if node.kind() == "state_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = node_text(&name_node, source).to_string();
            let mut states = Vec::new();
            let mut transitions = Vec::new();
            let mut entry_state = None;

            // Find state_body or definition_body
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "state_body" || child.kind() == "definition_body" {
                    extract_state_body(
                        &child,
                        source,
                        &mut states,
                        &mut transitions,
                        &mut entry_state,
                    );
                }
            }

            results.push(StateMachineModel {
                name,
                states,
                transitions,
                entry_state,
                span: Span::from_node(&node),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_state_defs(child, source, _file, results);
    }
}

fn extract_state_body(
    body: &Node,
    source: &[u8],
    states: &mut Vec<StateNode>,
    transitions: &mut Vec<Transition>,
    entry_state: &mut Option<String>,
) {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        match child.kind() {
            "state_usage" => {
                if let Some(state) = extract_state_node(&child, source) {
                    states.push(state);
                }
            }
            "entry_action" => {
                // entry; then StateName; — sets the initial state
                let mut ec = child.walk();
                let mut saw_then = false;
                for entry_child in child.children(&mut ec) {
                    if entry_child.kind() == "then" {
                        saw_then = true;
                    } else if saw_then
                        && (entry_child.kind() == "identifier"
                            || entry_child.kind() == "qualified_name")
                    {
                        *entry_state = Some(node_text(&entry_child, source).to_string());
                        break;
                    }
                }
            }
            "transition_statement" => {
                if let Some(t) = extract_transition(&child, source) {
                    transitions.push(t);
                }
            }
            "succession_usage" | "succession_statement" => {
                // first X then Y; — shorthand transition
                if let Some(t) = extract_succession_as_transition(&child, source) {
                    transitions.push(t);
                }
            }
            _ => {}
        }
    }
}

fn extract_state_node(node: &Node, source: &[u8]) -> Option<StateNode> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string())?;

    let mut entry_action = None;
    let mut do_action = None;
    let mut exit_action = None;

    // Walk children for state_body or definition_body
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "state_body" || child.kind() == "definition_body" {
            let mut bc = child.walk();
            for body_child in child.children(&mut bc) {
                match body_child.kind() {
                    "entry_action" => {
                        entry_action = extract_action_ref(&body_child, source);
                    }
                    "do_action" => {
                        do_action = extract_action_ref(&body_child, source);
                    }
                    "exit_action" => {
                        exit_action = extract_action_ref(&body_child, source);
                    }
                    _ => {}
                }
            }
        }
    }

    Some(StateNode {
        name,
        entry_action,
        do_action,
        exit_action,
        span: Span::from_node(node),
    })
}

fn extract_action_ref(node: &Node, source: &[u8]) -> Option<ActionRef> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "qualified_name" => {
                let text = node_text(&child, source).to_string();
                // Skip keywords like "entry", "do", "exit", "action", "then", "send"
                if !matches!(
                    text.as_str(),
                    "entry" | "do" | "exit" | "action" | "then" | "send"
                ) {
                    return Some(ActionRef::Named(text));
                }
            }
            "send_action" => {
                return Some(extract_send_action(&child, source));
            }
            _ => {}
        }
    }
    None
}

fn extract_send_action(node: &Node, source: &[u8]) -> ActionRef {
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

    ActionRef::Send { payload, via, to }
}

fn extract_transition(node: &Node, source: &[u8]) -> Option<Transition> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string());

    let mut source_state = None;
    let mut target_state = None;
    let mut trigger = None;
    let mut guard = None;
    let mut effect = None;

    let mut saw_first = false;
    let mut saw_if = false;
    let mut saw_do = false;
    let mut saw_then = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "first" => saw_first = true,
            "if" => saw_if = true,
            "do" => saw_do = true,
            "then" => saw_then = true,
            "accept_clause" => {
                trigger = extract_trigger(&child, source);
            }
            "identifier" | "qualified_name" | "feature_chain" => {
                let text = node_text(&child, source).to_string();
                if text == "transition" {
                    continue;
                }
                if saw_then {
                    target_state = Some(text);
                    saw_then = false;
                } else if saw_do {
                    effect = Some(ActionRef::Named(text));
                    saw_do = false;
                } else if saw_if {
                    // Guard is an expression — try to extract
                    if let Ok(expr) = extract_expr(&child, source) {
                        guard = Some(expr);
                    }
                    saw_if = false;
                } else if saw_first {
                    source_state = Some(text);
                    saw_first = false;
                }
            }
            _ => {
                // Try to extract guard from non-identifier expression nodes
                if saw_if && child.is_named() {
                    if let Ok(expr) = extract_expr(&child, source) {
                        guard = Some(expr);
                    }
                    saw_if = false;
                }
                if saw_do && child.is_named() {
                    effect = extract_action_ref(&child, source).or_else(|| {
                        Some(ActionRef::Inline(
                            node_text(&child, source).to_string(),
                        ))
                    });
                    saw_do = false;
                }
            }
        }
    }

    // Need at least source and target
    let src = source_state?;
    let tgt = target_state?;

    Some(Transition {
        name,
        source: src,
        target: tgt,
        trigger,
        guard,
        effect,
        span: Span::from_node(node),
    })
}

fn extract_trigger(node: &Node, source: &[u8]) -> Option<Trigger> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "qualified_name" => {
                let text = node_text(&child, source).to_string();
                if text != "accept" && text != "when" && text != "at" && text != "after" {
                    return Some(Trigger::Signal(text));
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_succession_as_transition(node: &Node, source: &[u8]) -> Option<Transition> {
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
        Some(Transition {
            name: None,
            source: refs[0].clone(),
            target: refs[1].clone(),
            trigger: None,
            guard: None,
            effect: None,
            span: Span::from_node(node),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_state_machine() {
        let source = r#"
            state def TrafficLight {
                entry; then red;
                state red;
                state yellow;
                state green;
                transition first red then green;
                transition first green then yellow;
                transition first yellow then red;
            }
        "#;
        let machines = extract_state_machines("test.sysml", source);
        assert_eq!(machines.len(), 1);
        let m = &machines[0];
        assert_eq!(m.name, "TrafficLight");
        assert_eq!(m.states.len(), 3);
        assert_eq!(m.entry_state, Some("red".to_string()));
        assert_eq!(m.transitions.len(), 3);
        assert_eq!(m.transitions[0].source, "red");
        assert_eq!(m.transitions[0].target, "green");
    }

    #[test]
    fn extract_flashlight_states() {
        let source = r#"
            state def FlashlightStates {
                entry; then off;
                state off;
                state on;
                transition off_to_on
                    first off
                    accept switchOn
                    then on;
                transition on_to_off
                    first on
                    accept switchOff
                    then off;
            }
        "#;
        let machines = extract_state_machines("test.sysml", source);
        assert_eq!(machines.len(), 1);
        let m = &machines[0];
        assert_eq!(m.states.len(), 2);
        assert_eq!(m.entry_state, Some("off".to_string()));
        assert_eq!(m.transitions.len(), 2);

        let t0 = &m.transitions[0];
        assert_eq!(t0.name, Some("off_to_on".to_string()));
        assert_eq!(t0.source, "off");
        assert_eq!(t0.target, "on");
        assert!(matches!(&t0.trigger, Some(Trigger::Signal(s)) if s == "switchOn"));

        let t1 = &m.transitions[1];
        assert_eq!(t1.source, "on");
        assert_eq!(t1.target, "off");
        assert!(matches!(&t1.trigger, Some(Trigger::Signal(s)) if s == "switchOff"));
    }

    #[test]
    fn no_state_machines_in_non_state_file() {
        let source = "part def Vehicle;";
        let machines = extract_state_machines("test.sysml", source);
        assert!(machines.is_empty());
    }
}
