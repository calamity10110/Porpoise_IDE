/// Platform detection via compile-time cfg constants.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    LinuxX64,
    LinuxArm64,
    MacOSX64,
    MacOSArm64,
    WindowsX64,
}

impl Platform {
    /// Returns the platform this binary was compiled for.
    pub const fn current() -> Self {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Self::LinuxX64;
        }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            return Self::LinuxArm64;
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            return Self::MacOSX64;
        }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            return Self::MacOSArm64;
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            Self::WindowsX64
        }
        // Fallback for other targets — shouldn't happen in practice
        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64"),
        )))]
        {
            compile_error!("unsupported target OS/arch combination for porpoise");
        }
    }

    /// True on any Unix-like OS (Linux, macOS).
    pub const fn is_unix(&self) -> bool {
        matches!(self, Self::LinuxX64 | Self::LinuxArm64 | Self::MacOSX64 | Self::MacOSArm64)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LinuxX64 => write!(f, "linux-x86_64"),
            Self::LinuxArm64 => write!(f, "linux-aarch64"),
            Self::MacOSX64 => write!(f, "macos-x86_64"),
            Self::MacOSArm64 => write!(f, "macos-aarch64"),
            Self::WindowsX64 => write!(f, "windows-x86_64"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform_is_unix() {
        let p = Platform::current();
        #[cfg(unix)]
        assert!(p.is_unix());
        #[cfg(windows)]
        assert!(!p.is_unix());
    }

    #[test]
    fn test_display() {
        let p = Platform::current();
        let s = p.to_string();
        assert!(!s.is_empty());
    }
}
