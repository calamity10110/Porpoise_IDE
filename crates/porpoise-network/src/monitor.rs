use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;

pub struct ConnectivityMonitor {
    connected: Arc<AtomicBool>,
    check_hosts: Vec<String>,
    interval: Duration,
}

impl Default for ConnectivityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectivityMonitor {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(AtomicBool::new(true)),
            check_hosts: vec![
                "8.8.8.8:53".into(),
                "1.1.1.1:53".into(),
            ],
            interval: Duration::from_secs(30),
        }
    }

    pub fn with_hosts(mut self, hosts: Vec<String>) -> Self {
        self.check_hosts = hosts;
        self
    }

    pub fn is_online(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub async fn run(&self) {
        let connected = self.connected.clone();
        let hosts = self.check_hosts.clone();
        let interval = self.interval;
        loop {
            let online = Self::check_connectivity(&hosts).await;
            connected.store(online, Ordering::Relaxed);
            tokio::time::sleep(interval).await;
        }
    }

    async fn check_connectivity(hosts: &[String]) -> bool {
        for host in hosts {
            if TcpStream::connect(host).await.is_ok() {
                return true;
            }
        }
        false
    }
}
