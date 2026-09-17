use porpoise_core::error::PorpoiseError;

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_cpu_seconds: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub max_fds: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_seconds: None,
            max_memory_bytes: None,
            max_fds: Some(1024),
        }
    }
}

impl ResourceLimits {
    pub fn apply(&self) -> std::result::Result<(), PorpoiseError> {
        #[cfg(unix)]
        {
            if let Some(fds) = self.max_fds {
                let rlim = libc::rlimit {
                    rlim_cur: fds,
                    rlim_max: fds,
                };
                // SAFETY: setrlimit sets the file descriptor limit for this process.
                // `rlim` is a valid stack-allocated rlimit struct with cur <= max.
                // The kernel validates the values and returns an error code on failure.
                let res = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };
                if res != 0 {
                    return Err(PorpoiseError::Runtime(format!("failed to set RLIMIT_NOFILE to {fds}")));
                }
            }
        }
        Ok(())
    }
}
