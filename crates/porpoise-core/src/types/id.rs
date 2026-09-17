use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{PorpoiseError, Result};

macro_rules! id_type {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn into_string(self) -> String {
                format!("{}{}", $prefix, self.0)
            }

            pub fn try_from_str(s: &str) -> Result<Self> {
                let inner = s
                    .strip_prefix($prefix)
                    .ok_or_else(|| PorpoiseError::invalid_id(stringify!($name), s))?;
                let uuid = Uuid::from_str(inner).map_err(|_| PorpoiseError::invalid_id(stringify!($name), s))?;
                Ok(Self(uuid))
            }

            pub fn is_valid_prefix(s: &str) -> bool {
                s.starts_with($prefix)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }

        impl FromStr for $name {
            type Err = PorpoiseError;

            fn from_str(s: &str) -> Result<Self> {
                Self::try_from_str(s)
            }
        }
    };
}

id_type!(WorktreeId, "wt_");
id_type!(TerminalId, "tm_");
id_type!(AgentId, "ag_");
id_type!(SessionId, "ss_");
id_type!(PageId, "pg_");
id_type!(CorrelationId, "cr_");
id_type!(ProcessId, "pr_");
id_type!(SkillId, "sk_");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_id_roundtrip() {
        let id = WorktreeId::new();
        let s = id.to_string();
        let parsed: WorktreeId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_terminal_id_roundtrip() {
        let id = TerminalId::new();
        let s = id.to_string();
        let parsed: TerminalId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_invalid_id() {
        let result: std::result::Result<WorktreeId, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_prefix() {
        let result: std::result::Result<WorktreeId, _> = "tm_1234".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_ids_are_unique() {
        let a = WorktreeId::new();
        let b = WorktreeId::new();
        assert_ne!(a, b);
    }
}