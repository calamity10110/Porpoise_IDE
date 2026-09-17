//! Property system — typed configuration for workflow nodes.
//!
//! Each node kind has a set of configurable properties. This module defines
//! the property schema, validation, and default values so the frontend can
//! render property editors dynamically.

use serde::{Deserialize, Serialize};

use crate::graph::{Node, NodeKind};

/// The data type of a property value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum PropType {
    /// Free-text string.
    String,
    /// Multi-line text.
    Text,
    /// Integer number.
    Integer,
    /// Floating-point number.
    Float,
    /// Boolean toggle.
    Boolean,
    /// Dropdown from a fixed set of choices.
    Select { options: Vec<String> },
    /// JSON value (advanced).
    Json,
}

/// A single property definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDef {
    /// Machine-readable key (matches `config` JSON key on the node).
    pub key: String,
    /// Human-readable label.
    pub label: String,
    /// Data type.
    pub prop_type: PropType,
    /// Default value (JSON).
    pub default: serde_json::Value,
    /// Whether the property is required.
    pub required: bool,
    /// Short help text.
    pub help: Option<String>,
}

/// Get the property schema for a given node kind.
pub fn props_for_kind(kind: &NodeKind) -> Vec<PropDef> {
    match kind {
        NodeKind::Agent { .. } => vec![
            PropDef {
                key: "agent_kind".into(),
                label: "Agent".into(),
                prop_type: PropType::Select {
                    options: vec!["claude".into(), "codex".into(), "aider".into(), "opencode".into()],
                },
                default: serde_json::json!("claude"),
                required: true,
                help: Some("Which agent adapter to use.".into()),
            },
            PropDef {
                key: "prompt".into(),
                label: "System Prompt".into(),
                prop_type: PropType::Text,
                default: serde_json::json!(null),
                required: false,
                help: Some("Optional system prompt override.".into()),
            },
            PropDef {
                key: "timeout_secs".into(),
                label: "Timeout (s)".into(),
                prop_type: PropType::Integer,
                default: serde_json::json!(300),
                required: false,
                help: Some("Max execution time in seconds.".into()),
            },
        ],
        NodeKind::Command { .. } => vec![
            PropDef {
                key: "command".into(),
                label: "Command".into(),
                prop_type: PropType::String,
                default: serde_json::json!(""),
                required: true,
                help: Some("Shell command to execute.".into()),
            },
            PropDef {
                key: "working_dir".into(),
                label: "Working Directory".into(),
                prop_type: PropType::String,
                default: serde_json::json!(null),
                required: false,
                help: Some("Override working directory.".into()),
            },
        ],
        NodeKind::Condition { .. } => vec![PropDef {
            key: "expression".into(),
            label: "Expression".into(),
            prop_type: PropType::String,
            default: serde_json::json!(""),
            required: true,
            help: Some("Boolean expression to evaluate.".into()),
        }],
        NodeKind::Merge | NodeKind::Start | NodeKind::End => vec![],
    }
}

/// Validate a node's `config` against its property schema.
/// Returns a list of error messages (empty = valid).
pub fn validate_node_props(node: &Node) -> Vec<String> {
    let defs = props_for_kind(&node.kind);
    let mut errors = Vec::new();

    for def in &defs {
        let value = node.config.get(&def.key);
        if def.required && (value.is_none() || value == Some(&serde_json::Value::Null)) {
            errors.push(format!("Required property '{}' is missing", def.label));
        }
        if let Some(v) = value {
            match &def.prop_type {
                PropType::Integer if !v.is_i64() && !v.is_u64() => {
                    errors.push(format!("Property '{}' must be an integer", def.label));
                }
                PropType::Float if !v.is_f64() && !v.is_i64() && !v.is_u64() => {
                    errors.push(format!("Property '{}' must be a number", def.label));
                }
                PropType::Boolean if !v.is_boolean() => {
                    errors.push(format!("Property '{}' must be a boolean", def.label));
                }
                PropType::Select { options } => {
                    if let Some(s) = v.as_str() {
                        if !options.iter().any(|o| o == s) {
                            errors.push(format!(
                                "Property '{}' must be one of: {}",
                                def.label,
                                options.join(", ")
                            ));
                        }
                    } else {
                        errors.push(format!("Property '{}' must be a string", def.label));
                    }
                }
                _ => {}
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Position;

    fn make_node(kind: NodeKind, config: serde_json::Value) -> Node {
        Node {
            id: "test".into(),
            label: "Test".into(),
            kind,
            position: Position { x: 0.0, y: 0.0 },
            config,
        }
    }

    #[test]
    fn agent_props_count() {
        let props = props_for_kind(&NodeKind::Agent {
            agent_kind: "claude".into(),
            prompt: None,
        });
        assert_eq!(props.len(), 3);
    }

    #[test]
    fn merge_has_no_props() {
        let props = props_for_kind(&NodeKind::Merge);
        assert!(props.is_empty());
    }

    #[test]
    fn validate_missing_required() {
        let node = make_node(NodeKind::Command { command: String::new() }, serde_json::json!({}));
        let errors = validate_node_props(&node);
        assert!(errors.iter().any(|e| e.contains("Command")));
    }

    #[test]
    fn validate_valid_command() {
        let node = make_node(
            NodeKind::Command {
                command: "echo hello".into(),
            },
            serde_json::json!({"command": "echo hello"}),
        );
        let errors = validate_node_props(&node);
        assert!(errors.is_empty());
    }

    #[test]
    fn validate_bad_select() {
        let node = make_node(
            NodeKind::Agent {
                agent_kind: "claude".into(),
                prompt: None,
            },
            serde_json::json!({"agent_kind": "invalid_agent"}),
        );
        let errors = validate_node_props(&node);
        assert!(errors.iter().any(|e| e.contains("one of")));
    }
}
