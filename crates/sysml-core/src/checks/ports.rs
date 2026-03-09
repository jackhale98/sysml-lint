/// Checks for port type compatibility on connections.

use std::collections::HashMap;

use crate::checks::Check;
use crate::diagnostic::{codes, Diagnostic};
use crate::model::{simple_name, Model};

pub struct PortConnectionCheck;

impl Check for PortConnectionCheck {
    fn name(&self) -> &'static str {
        "port-types"
    }

    fn run(&self, model: &Model) -> Vec<Diagnostic> {
        // Build port name -> type mapping from port usages
        let mut port_types: HashMap<&str, &str> = HashMap::new();
        for u in &model.usages {
            if u.kind == "port" {
                if let Some(ref t) = u.type_ref {
                    port_types.insert(u.name.as_str(), t.as_str());
                }
            }
        }

        let mut diagnostics = Vec::new();

        for conn in &model.connections {
            // Extract port names from dot-paths (e.g., "engine.fuelPort" -> "fuelPort")
            let src_port = simple_name(&conn.source);
            let tgt_port = simple_name(&conn.target);

            let src_type = port_types.get(src_port);
            let tgt_type = port_types.get(tgt_port);

            if let (Some(&st), Some(&tt)) = (src_type, tgt_type) {
                // Check type compatibility
                // Types are compatible if they are the same, or one is conjugated (~)
                let st_base = st.strip_prefix('~').unwrap_or(st);
                let tt_base = tt.strip_prefix('~').unwrap_or(tt);

                if simple_name(st_base) != simple_name(tt_base) {
                    diagnostics.push(Diagnostic::warning(
                        &model.file,
                        conn.span.clone(),
                        codes::PORT_TYPE_MISMATCH,
                        format!(
                            "connected ports have different types: `{}` is `{}` but `{}` is `{}`",
                            src_port, st, tgt_port, tt,
                        ),
                    ));
                }
            }
        }

        diagnostics
    }
}
