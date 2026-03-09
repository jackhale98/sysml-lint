use std::process::ExitCode;

struct Topic {
    slug: &'static str,
    title: &'static str,
    body: &'static str,
}

const TOPICS: &[Topic] = &[
    Topic {
        slug: "getting-started",
        title: "Getting Started with sysml",
        body: "\
First, install sysml using your preferred method (cargo install sysml-cli, or \
download a release binary). Once installed, create a plain text file with a .sysml \
extension — for example, vehicle.sysml — and write a simple definition:

    part def Vehicle {
        doc /* A road vehicle with an engine and wheels. */
        part engine : Engine;
        part wheels : Wheel;
    }

    part def Engine;
    part def Wheel;

Now validate your model by running:

    sysml lint vehicle.sysml

This checks for syntax errors, duplicate definitions, unused elements, and other \
structural issues. Fix any reported problems, then explore your model:

    sysml list vehicle.sysml              # see all definitions and usages
    sysml show vehicle.sysml Vehicle      # inspect a specific element
    sysml diagram -t bdd vehicle.sysml    # generate a block definition diagram

The diagram command outputs Mermaid text by default, which you can paste into \
GitHub, Obsidian, or any Mermaid-compatible viewer to see a visual representation \
of your model. From here, try adding requirements, state machines, or constraints \
and use the other commands to analyze and simulate your design.",
    },
    Topic {
        slug: "mbse",
        title: "What Is Model-Based Systems Engineering?",
        body: "\
Model-Based Systems Engineering (MBSE) replaces traditional document-centric \
engineering with a shared, structured model that serves as the single source of \
truth for a system's design. Instead of maintaining separate requirements \
documents, interface spreadsheets, and architecture diagrams that can fall out \
of sync, MBSE captures all of this information in one interconnected model.

The practical benefit is traceability: every requirement links to the design \
element that satisfies it, every interface has a defined type, and every test \
case traces back to what it verifies. When a change is made, its impact is \
immediately visible. SysML v2 is the standard language for writing these \
models, and sysml is a command-line tool that lets you validate, query, \
simulate, and visualize SysML v2 models without needing a heavyweight \
graphical tool.",
    },
    Topic {
        slug: "sysml-basics",
        title: "SysML v2 Language Overview",
        body: "\
SysML v2 uses a textual notation built around two core concepts: definitions \
and usages. A definition (part def, port def, action def, etc.) declares a \
reusable type — think of it as a blueprint. A usage (part, port, action, etc.) \
creates an instance of that type within a specific context. For example, \
\"part def Engine\" defines what an engine is, while \"part engine : Engine\" \
creates a specific engine inside another definition.

Relationships connect elements together. Specialization (:>) says one \
definition extends another (\"part def ElectricEngine :> Engine\"). Connections \
link ports between parts to describe interfaces. Satisfy and verify \
relationships trace requirements to design elements and test cases. Packages \
group related definitions, and imports bring elements from one package into \
another. This small set of concepts — definitions, usages, relationships, and \
packages — is enough to describe complex systems from requirements through \
detailed design.",
    },
    Topic {
        slug: "requirements",
        title: "Requirements Management in SysML v2",
        body: "\
In SysML v2, requirements are first-class model elements rather than rows in a \
spreadsheet. You define them with \"requirement def\" and give each one a text \
description and optionally a unique identifier. Requirements can be organized \
into hierarchies using specialization, so a top-level system requirement can \
have child requirements that refine it.

Traceability comes from two key relationships. A \"satisfy\" relationship links \
a design element (like a part definition) to the requirement it addresses. A \
\"verify\" relationship links a test case or verification activity to the \
requirement it checks. Run \"sysml trace\" to see a traceability matrix showing \
which requirements are satisfied, which are verified, and which have gaps. Use \
\"sysml trace --check\" in your CI pipeline to ensure no requirement is left \
unaddressed.",
    },
    Topic {
        slug: "verification",
        title: "Verification and Validation Concepts",
        body: "\
Verification asks \"did we build the system right?\" — it confirms the design \
meets its specified requirements. Validation asks \"did we build the right \
system?\" — it confirms the system meets the stakeholder's actual needs. In \
SysML v2, verification is modeled explicitly: you write verification cases \
that reference the requirements they check, creating a traceable chain from \
stakeholder needs through requirements to test evidence.

With sysml, you can check verification coverage by running \"sysml trace\" to \
see which requirements have verify relationships and which do not. The \
\"sysml coverage\" command goes further and computes an overall quality score \
that includes verification status along with documentation completeness and \
type coverage. Use \"sysml coverage --check --min-score 80\" in continuous \
integration to enforce a minimum standard across your model.",
    },
    Topic {
        slug: "diagrams",
        title: "Available Diagram Types",
        body: "\
The \"sysml diagram\" command generates seven diagram types, each suited to a \
different modeling question. BDD (Block Definition Diagram) shows definitions \
and their relationships — use it for an overview of your system's types. IBD \
(Internal Block Diagram) shows the internal structure of a single part, \
including its ports, nested parts, and connections — use it to understand \
interfaces. STM (State Machine Diagram) visualizes states and transitions — \
use it for behavior that depends on events. ACT (Activity Diagram) shows \
action flows with decisions, forks, and loops — use it for processes and \
workflows.

REQ (Requirements Diagram) displays requirements and their satisfy/verify \
relationships — use it for traceability reviews. PKG (Package Diagram) shows \
the package hierarchy and imports — use it to understand model organization. \
PAR (Parametric Diagram) shows constraint blocks and their parameters — use \
it for engineering analysis relationships. All diagrams can be output in \
Mermaid, PlantUML, DOT (Graphviz), or D2 format using the -o flag.",
    },
    Topic {
        slug: "projects",
        title: "Setting Up a SysML Project",
        body: "\
A SysML project is simply a directory containing .sysml files. There is no \
special project file or configuration required — sysml operates directly on \
the files you point it at. For a clean layout, consider organizing your model \
into subdirectories by concern: requirements/, architecture/, behavior/, and \
interfaces/. Each file can define one or more packages, and you use SysML \
import statements to reference definitions across files.

When running commands on multi-file projects, pass all relevant files (or \
directories, which are searched recursively) so that cross-file references \
resolve correctly. You can also use the -I flag to add include directories \
for shared libraries. For example: \"sysml lint -I shared/ src/\" validates \
everything in src/ with definitions from shared/ available for import \
resolution. In CI, combine \"sysml lint\", \"sysml trace --check\", and \
\"sysml coverage --check\" to enforce model quality on every commit.",
    },
    Topic {
        slug: "commands",
        title: "Quick Reference of All Commands",
        body: "\
    sysml lint <files>              Validate models against structural rules
    sysml list <files>              List definitions and usages with filters
    sysml show <file> <name>        Inspect a single element in detail
    sysml trace <files>             Requirements traceability matrix
    sysml interfaces <files>        Port analysis and unconnected port detection
    sysml diagram -t <type> <file>  Generate BDD, IBD, STM, ACT, REQ, PKG, or PAR
    sysml simulate eval <file>      Evaluate constraints and calculations
    sysml simulate sm <file>        Simulate a state machine step-by-step
    sysml simulate af <file>        Execute an action flow
    sysml export interfaces <file>  Extract FMI interface descriptions
    sysml export modelica <file>    Generate Modelica model stubs
    sysml export ssp <file>         Generate SSP system structure XML
    sysml new <kind> <name>         Generate a definition template to stdout
    sysml edit add <file> ...       Add an element to an existing file
    sysml edit remove <file> <name> Remove an element from a file
    sysml edit rename <file> ...    Rename an element and update references
    sysml fmt <files>               Format SysML files
    sysml stats <files>             Model statistics and metrics
    sysml deps <files> <name>       Dependency and impact analysis
    sysml diff <old> <new>          Semantic diff between two model files
    sysml allocation <files>        Logical-to-physical allocation matrix
    sysml coverage <files>          Model completeness and quality score
    sysml completions <shell>       Generate shell completions
    sysml help <topic>              Read a help topic (this command)

Run \"sysml <command> --help\" for detailed usage of any command.",
    },
];

pub fn run(topic: Option<&str>) -> ExitCode {
    let topic_slug = match topic {
        Some(t) => t,
        None => {
            println!("Available help topics:\n");
            for t in TOPICS {
                println!("  {:<20} {}", t.slug, t.title);
            }
            println!("\nRun \"sysml help <topic>\" to read a topic.");
            return ExitCode::SUCCESS;
        }
    };

    for t in TOPICS {
        if t.slug == topic_slug {
            println!("{}\n", t.title);
            println!("{}", t.body);
            return ExitCode::SUCCESS;
        }
    }

    eprintln!("error: unknown help topic `{}`\n", topic_slug);
    eprintln!("Available topics:");
    for t in TOPICS {
        eprintln!("  {:<20} {}", t.slug, t.title);
    }
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_topics_returns_success() {
        assert_eq!(run(None), ExitCode::SUCCESS);
    }

    #[test]
    fn known_topic_returns_success() {
        for t in TOPICS {
            assert_eq!(run(Some(t.slug)), ExitCode::SUCCESS);
        }
    }

    #[test]
    fn unknown_topic_returns_failure() {
        assert_eq!(run(Some("nonexistent-topic")), ExitCode::FAILURE);
    }
}
