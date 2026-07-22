use std::path::{Path, PathBuf};

use porpoise_core::error::{PorpoiseError, Result};
use rcgen::{CertificateParams, KeyPair};

pub struct TlsAssets {
    pub cert_pem: Vec<u8>,
    pub key_pem: Vec<u8>,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub fingerprint_sha256: String,
}

pub fn load_or_generate(data_dir: &Path, hostnames: &[&str]) -> Result<TlsAssets> {
    let cert_path = data_dir.join("daemon.cert.pem");
    let key_path = data_dir.join("daemon.key.pem");
    let fp_path = data_dir.join("daemon.cert.fp");

    if cert_path.exists() && key_path.exists() && fp_path.exists() {
        let cert_pem = std::fs::read(&cert_path).map_err(|e| PorpoiseError::Ipc(format!("read cert: {e}")))?;
        let key_pem = std::fs::read(&key_path).map_err(|e| PorpoiseError::Ipc(format!("read key: {e}")))?;
        let fingerprint = std::fs::read_to_string(&fp_path)
            .map_err(|e| PorpoiseError::Ipc(format!("read fp: {e}")))?
            .trim()
            .to_string();
        return Ok(TlsAssets {
            cert_pem,
            key_pem,
            cert_path,
            key_path,
            fingerprint_sha256: fingerprint,
        });
    }

    generate_and_store(&cert_path, &key_path, &fp_path, hostnames)
}

fn generate_and_store(cert_path: &Path, key_path: &Path, fp_path: &Path, hostnames: &[&str]) -> Result<TlsAssets> {
    let san: Vec<String> = hostnames.iter().map(|s| s.to_string()).collect();
    let mut params = CertificateParams::new(san).map_err(|e| PorpoiseError::Ipc(format!("cert params: {e}")))?;
    params.distinguished_name = rcgen::DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "Porpoise Daemon");
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationName, "Porpoise");

    let key_pair = KeyPair::generate().map_err(|e| PorpoiseError::Ipc(format!("keypair: {e}")))?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| PorpoiseError::Ipc(format!("self_signed: {e}")))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    let cert_der = cert.der().to_vec();

    let fingerprint = sha256_hex(&cert_der);

    write_restricted(cert_path, cert_pem.as_bytes())?;
    write_restricted(key_path, key_pem.as_bytes())?;
    write_restricted(fp_path, fingerprint.as_bytes())?;

    Ok(TlsAssets {
        cert_pem: cert_pem.into_bytes(),
        key_pem: key_pem.into_bytes(),
        cert_path: cert_path.to_path_buf(),
        key_path: key_path.to_path_buf(),
        fingerprint_sha256: fingerprint,
    })
}

fn write_restricted(path: &Path, content: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| PorpoiseError::Ipc(format!("create {path:?}: {e}")))?;
        file.write_all(content)
            .map_err(|e| PorpoiseError::Ipc(format!("write {path:?}: {e}")))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, content).map_err(|e| PorpoiseError::Ipc(format!("write {path:?}: {e}")))?;
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn build_tls_acceptor(assets: &TlsAssets) -> Result<tokio_rustls::TlsAcceptor> {
    use std::sync::Arc;

    use tokio_rustls::rustls::ServerConfig;

    let provider = Arc::new(tokio_rustls::rustls::crypto::ring::default_provider());

    let certs = rustls_pemfile::certs(&mut assets.cert_pem.as_slice())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| PorpoiseError::Ipc(format!("parse certs: {e}")))?;
    if certs.is_empty() {
        return Err(PorpoiseError::Ipc("no certificates found in PEM".into()));
    }

    let key = rustls_pemfile::private_key(&mut assets.key_pem.as_slice())
        .map_err(|e| PorpoiseError::Ipc(format!("parse key: {e}")))?
        .ok_or_else(|| PorpoiseError::Ipc("no private key in PEM".into()))?;

    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| PorpoiseError::Ipc(format!("tls versions: {e}")))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| PorpoiseError::Ipc(format!("tls config: {e}")))?;

    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_load() {
        let dir = tempfile::TempDir::new().unwrap();
        let assets1 = load_or_generate(dir.path(), &["localhost", "127.0.0.1"]).unwrap();
        assert!(!assets1.fingerprint_sha256.is_empty());

        let assets2 = load_or_generate(dir.path(), &["localhost"]).unwrap();
        assert_eq!(assets1.fingerprint_sha256, assets2.fingerprint_sha256);
    }

    #[test]
    fn test_build_acceptor() {
        let dir = tempfile::TempDir::new().unwrap();
        let assets = load_or_generate(dir.path(), &["localhost"]).unwrap();
        let acceptor = build_tls_acceptor(&assets);
        assert!(acceptor.is_ok(), "acceptor build failed: {:?}", acceptor.err());
    }

    #[test]
    fn test_fingerprint_is_64_hex_chars() {
        let dir = tempfile::TempDir::new().unwrap();
        let assets = load_or_generate(dir.path(), &["localhost"]).unwrap();
        assert_eq!(assets.fingerprint_sha256.len(), 64);
        assert!(assets.fingerprint_sha256.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
