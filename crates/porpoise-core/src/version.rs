//! Application version tracking, parsed from Cargo.toml at compile time.

use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

use crate::error::PorpoiseError;

/// Application version, lazily parsed from `CARGO_PKG_VERSION` at first access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: &'static str,
}

impl AppVersion {
    /// Returns the version from `CARGO_PKG_VERSION` (e.g. "0.1.0").
    /// Parsed once and cached in a static.
    pub fn current() -> &'static Self {
        static V: OnceLock<AppVersion> = OnceLock::new();
        V.get_or_init(Self::parse_from_env)
    }

    fn parse_from_env() -> Self {
        let version = env!("CARGO_PKG_VERSION");
        let (num, pre) = match version.split_once('-') {
            Some((n, p)) => (n, p),
            None => (version, ""),
        };
        let mut parts = num.split('.');
        let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let pre: &'static str = if pre.is_empty() {
            ""
        } else {
            // Leak is intentional: runs once, version is a compile-time constant.
            Box::leak(pre.to_string().into_boxed_str())
        };
        Self { major, minor, patch, pre }
    }
}

impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        Ok(())
    }
}

impl FromStr for AppVersion {
    type Err = PorpoiseError;

    fn from_str(s: &str) -> std::result::Result<Self, PorpoiseError> {
        let (num, pre) = match s.split_once('-') {
            Some((n, p)) => (n, p),
            None => (s, ""),
        };
        let mut parts = num.split('.');
        let major = parts
            .next()
            .ok_or_else(|| PorpoiseError::Validation("missing major".into()))?
            .parse()
            .map_err(|_| PorpoiseError::Validation("invalid major".into()))?;
        let minor = parts
            .next()
            .ok_or_else(|| PorpoiseError::Validation("missing minor".into()))?
            .parse()
            .map_err(|_| PorpoiseError::Validation("invalid minor".into()))?;
        let patch = parts
            .next()
            .ok_or_else(|| PorpoiseError::Validation("missing patch".into()))?
            .parse()
            .map_err(|_| PorpoiseError::Validation("invalid patch".into()))?;
        let pre: &'static str = if pre.is_empty() {
            ""
        } else {
            Box::leak(pre.to_string().into_boxed_str())
        };
        Ok(Self { major, minor, patch, pre })
    }
}

impl serde::Serialize for AppVersion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for AppVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_parse() {
        let v = AppVersion::current();
        assert_eq!(v.major, 0);
        assert!(!v.to_string().is_empty());
    }

    #[test]
    fn test_display_version() {
        assert_eq!(AppVersion { major: 1, minor: 2, patch: 3, pre: "" }.to_string(), "1.2.3");
        assert_eq!(AppVersion { major: 0, minor: 1, patch: 0, pre: "rc1" }.to_string(), "0.1.0-rc1");
    }

    #[test]
    fn test_from_str() {
        let v: AppVersion = "1.2.3".parse().unwrap();
        assert_eq!(v, AppVersion { major: 1, minor: 2, patch: 3, pre: "" });
    }

    #[test]
    fn test_from_str_pre() {
        let v: AppVersion = "0.1.0-beta.2".parse().unwrap();
        assert_eq!(v.pre, "beta.2");
    }
}
