use zeroize::Zeroizing;

pub enum AuthMethod {
    Password(Zeroizing<String>),
    KeyFile(String, Option<Zeroizing<String>>),
    Agent,
}

impl AuthMethod {
    pub fn password(p: impl Into<String>) -> Self {
        Self::Password(Zeroizing::new(p.into()))
    }

    pub fn key_file(path: impl Into<String>, passphrase: Option<String>) -> Self {
        Self::KeyFile(path.into(), passphrase.map(Zeroizing::new))
    }
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
