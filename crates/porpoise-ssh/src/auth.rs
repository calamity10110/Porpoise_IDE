pub enum AuthMethod {
    Password(String),
    KeyFile(String, Option<String>),
    Agent,
}

impl std::fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Password(_) => write!(f, "Password([redacted])"),
            AuthMethod::KeyFile(path, _) => write!(f, "KeyFile({})", path),
            AuthMethod::Agent => write!(f, "Agent"),
        }
    }
}
