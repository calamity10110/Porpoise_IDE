//! Component library — reusable visual building blocks for the design canvas.
//!
//! Each component maps a `NodeKind` to a visual template (icon, color, default
//! label, input/output port definitions). The frontend uses these definitions
//! to render consistent node cards.

use serde::{Deserialize, Serialize};

use crate::graph::NodeKind;

/// Color in `#RRGGBB` format.
pub type Color = String;

/// A port on a node (input or output connection point).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Port {
    /// Unique port identifier within the node.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Whether this port accepts multiple connections.
    pub multi: bool,
}

/// Visual definition for a node kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDef {
    /// Node kind this component represents.
    pub kind_tag: String,
    /// Display name.
    pub display_name: String,
    /// Icon identifier (frontend icon set key).
    pub icon: String,
    /// Background color.
    pub color: Color,
    /// Input ports.
    pub inputs: Vec<Port>,
    /// Output ports.
    pub outputs: Vec<Port>,
    /// Default label template (may contain `{agent_kind}`, `{command}`, etc.).
    pub default_label: String,
}

impl ComponentDef {
    /// Build a `ComponentDef` for a given `NodeKind`.
    pub fn for_kind(kind: &NodeKind) -> Self {
        match kind {
            NodeKind::Agent { agent_kind, .. } => Self {
                kind_tag: "Agent".into(),
                display_name: format!("Agent: {}", agent_kind),
                icon: "robot".into(),
                color: "#4A90D9".into(),
                inputs: vec![Port {
                    id: "in".into(),
                    label: "Input".into(),
                    multi: false,
                }],
                outputs: vec![
                    Port {
                        id: "success".into(),
                        label: "Success".into(),
                        multi: false,
                    },
                    Port {
                        id: "error".into(),
                        label: "Error".into(),
                        multi: false,
                    },
                ],
                default_label: format!("Agent ({})", agent_kind),
            },
            NodeKind::Command { .. } => Self {
                kind_tag: "Command".into(),
                display_name: "Shell Command".into(),
                icon: "terminal".into(),
                color: "#50C878".into(),
                inputs: vec![Port {
                    id: "in".into(),
                    label: "Input".into(),
                    multi: false,
                }],
                outputs: vec![
                    Port {
                        id: "stdout".into(),
                        label: "Stdout".into(),
                        multi: false,
                    },
                    Port {
                        id: "stderr".into(),
                        label: "Stderr".into(),
                        multi: false,
                    },
                ],
                default_label: "Command".into(),
            },
            NodeKind::Condition { .. } => Self {
                kind_tag: "Condition".into(),
                display_name: "Condition".into(),
                icon: "git-branch".into(),
                color: "#F5A623".into(),
                inputs: vec![Port {
                    id: "in".into(),
                    label: "Input".into(),
                    multi: false,
                }],
                outputs: vec![
                    Port {
                        id: "true".into(),
                        label: "True".into(),
                        multi: false,
                    },
                    Port {
                        id: "false".into(),
                        label: "False".into(),
                        multi: false,
                    },
                ],
                default_label: "Condition".into(),
            },
            NodeKind::Merge => Self {
                kind_tag: "Merge".into(),
                display_name: "Merge".into(),
                icon: "git-merge".into(),
                color: "#9B59B6".into(),
                inputs: vec![Port {
                    id: "in".into(),
                    label: "Inputs".into(),
                    multi: true,
                }],
                outputs: vec![Port {
                    id: "out".into(),
                    label: "Output".into(),
                    multi: false,
                }],
                default_label: "Merge".into(),
            },
            NodeKind::Start => Self {
                kind_tag: "Start".into(),
                display_name: "Start".into(),
                icon: "play".into(),
                color: "#2ECC71".into(),
                inputs: vec![],
                outputs: vec![Port {
                    id: "out".into(),
                    label: "Next".into(),
                    multi: false,
                }],
                default_label: "Start".into(),
            },
            NodeKind::End => Self {
                kind_tag: "End".into(),
                display_name: "End".into(),
                icon: "stop".into(),
                color: "#E74C3C".into(),
                inputs: vec![Port {
                    id: "in".into(),
                    label: "Done".into(),
                    multi: false,
                }],
                outputs: vec![],
                default_label: "End".into(),
            },
        }
    }
}

/// Get all built-in component definitions.
pub fn builtin_components() -> Vec<ComponentDef> {
    vec![
        ComponentDef::for_kind(&NodeKind::Start),
        ComponentDef::for_kind(&NodeKind::End),
        ComponentDef::for_kind(&NodeKind::Agent {
            agent_kind: "claude".into(),
            prompt: None,
        }),
        ComponentDef::for_kind(&NodeKind::Command { command: String::new() }),
        ComponentDef::for_kind(&NodeKind::Condition {
            expression: String::new(),
        }),
        ComponentDef::for_kind(&NodeKind::Merge),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_component_has_two_outputs() {
        let def = ComponentDef::for_kind(&NodeKind::Agent {
            agent_kind: "codex".into(),
            prompt: None,
        });
        assert_eq!(def.outputs.len(), 2);
        assert_eq!(def.outputs[0].id, "success");
        assert_eq!(def.outputs[1].id, "error");
    }

    #[test]
    fn start_has_no_inputs() {
        let def = ComponentDef::for_kind(&NodeKind::Start);
        assert!(def.inputs.is_empty());
        assert_eq!(def.outputs.len(), 1);
    }

    #[test]
    fn end_has_no_outputs() {
        let def = ComponentDef::for_kind(&NodeKind::End);
        assert!(def.outputs.is_empty());
        assert_eq!(def.inputs.len(), 1);
    }

    #[test]
    fn merge_accepts_multi_input() {
        let def = ComponentDef::for_kind(&NodeKind::Merge);
        assert!(def.inputs[0].multi);
    }

    #[test]
    fn builtin_count() {
        assert_eq!(builtin_components().len(), 6);
    }
}
