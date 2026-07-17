use std::{collections::HashMap, sync::Arc};

use porpoise_core::{
    bus::EventBus,
    error::{PorpoiseError, Result},
    types::id::TerminalId,
};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct PtySession {
    pub id: TerminalId,
    pub fd: i32,
    pub child_pid: u32,
    pub rows: u16,
    pub cols: u16,
    pub generation_id: Option<String>,
}

impl PtySession {
    pub fn master_fd(&self) -> i32 {
        self.fd
    }
}

#[cfg(not(target_os = "windows"))]
fn alloc_pty_impl(rows: u16, cols: u16, shell: &str) -> Result<PtySession> {
    use nix::pty::{self, Winsize};
    use nix::unistd::{self, ForkResult};
    use std::os::unix::io::AsRawFd;

    let winsize = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
    let (ptym, ptys) = pty::openpty(&winsize, None).map_err(|e| PorpoiseError::PtyError(e.to_string()))?;

    let master_fd = ptym.as_raw_fd();
    let slave_fd = ptys.as_raw_fd();

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child }) => {
            unistd::close(slave_fd).ok();
            Ok(PtySession { id: TerminalId::new(), fd: master_fd, child_pid: child.as_raw(), rows, cols, generation_id: None })
        }
        Ok(ForkResult::Child) => {
            unistd::setsid().ok();
            unistd::close(master_fd).ok();
            unsafe {
                libc::dup2(slave_fd, 0);
                libc::dup2(slave_fd, 1);
                libc::dup2(slave_fd, 2);
            }
            if slave_fd > 2 { unistd::close(slave_fd).ok(); }
            unistd::execvp(shell, &[shell]).ok();
            std::process::exit(1);
        }
        Err(e) => Err(PorpoiseError::PtyError(e.to_string())),
    }
}

#[cfg(target_os = "windows")]
mod windows_pty {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use portable_pty::MasterPty;
    use std::sync::LazyLock;

    static WINDOWS_PTYS: LazyLock<Mutex<HashMap<i32, Box<dyn MasterPty + Send>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    pub fn store_master(fd: i32, master: Box<dyn MasterPty + Send>) {
        WINDOWS_PTYS.lock().unwrap().insert(fd, master);
    }

    pub fn remove_master(fd: i32) {
        WINDOWS_PTYS.lock().unwrap().remove(&fd);
    }

    pub fn with_master<R>(fd: i32, f: impl FnOnce(&mut dyn MasterPty) -> R) -> R {
        let mut map = WINDOWS_PTYS.lock().unwrap();
        let master = map.get_mut(&fd).expect("master pty not found");
        f(&mut **master)
    }
}

#[cfg(target_os = "windows")]
fn alloc_pty_impl(rows: u16, cols: u16, shell: &str) -> Result<PtySession> {
    use portable_pty::{native_pty_system, PtySize, CommandBuilder};
    use std::sync::atomic::{AtomicI32, Ordering};

    static NEXT_FD: AtomicI32 = AtomicI32::new(1000);

    let pty_system = native_pty_system();
    let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
    let pair = pty_system.openpty(size).map_err(|e| PorpoiseError::PtyError(format!("openpty: {e}")))?;

    let cmd = CommandBuilder::new(shell);
    let child = pair.slave.spawn_command(cmd).map_err(|e| PorpoiseError::PtyError(format!("spawn: {e}")))?;

    let pid = child.process_id().unwrap_or(0);
    let fd = NEXT_FD.fetch_add(1, Ordering::Relaxed);

    windows_pty::store_master(fd, pair.master);

    Ok(PtySession { id: TerminalId::new(), fd, child_pid: pid, rows, cols, generation_id: None })
}

async fn pty_read_impl(fd: i32, buf: &mut [u8]) -> Result<usize> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::io::FromRawFd;
        use tokio::io::AsyncReadExt;
        // dup() creates a copy of fd so the original stays valid
        let dup_fd = unsafe { libc::dup(fd) };
        if dup_fd < 0 {
            return Err(PorpoiseError::PtyError("dup failed".into()));
        }
        let std_file = unsafe { std::fs::File::from_raw_fd(dup_fd) };
        let mut file = tokio::fs::File::from_std(std_file);
        file.read(buf).await.map_err(|e| PorpoiseError::PtyError(e.to_string()))
    }
    #[cfg(target_os = "windows")]
    {
        use std::io::Read;
        let count = tokio::task::spawn_blocking(move || -> std::io::Result<Vec<u8>> {
            windows_pty::with_master(fd, |master| {
                let mut reader = master.try_clone_reader().expect("try_clone_reader failed");
                let mut read_buf = vec![0u8; 4096];
                let n = reader.read(&mut read_buf)?;
                read_buf.truncate(n);
                Ok(read_buf)
            })
        })
        .await
        .map_err(|e| PorpoiseError::PtyError(format!("blocking read: {e}")))?
        .map_err(|e| PorpoiseError::PtyError(format!("read: {e}")))?;
        let n = count.len();
        if n > 0 {
            buf[..n].copy_from_slice(&count[..n]);
        }
        Ok(n)
    }
}

async fn pty_write_impl(fd: i32, data: &[u8]) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::io::FromRawFd;
        use tokio::io::AsyncWriteExt;
        let dup_fd = unsafe { libc::dup(fd) };
        if dup_fd < 0 {
            return Err(PorpoiseError::PtyError("dup failed".into()));
        }
        let std_file = unsafe { std::fs::File::from_raw_fd(dup_fd) };
        let mut file = tokio::fs::File::from_std(std_file);
        file.write_all(data).await.map_err(|e| PorpoiseError::PtyError(e.to_string()))
    }
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        let data = data.to_vec();
        tokio::task::spawn_blocking(move || {
            windows_pty::with_master(fd, |master| {
                let mut writer = master.take_writer().expect("take_writer failed");
                writer.write_all(&data)
            })
        })
        .await
        .map_err(|e| PorpoiseError::PtyError(format!("blocking: {e}")))?
        .map_err(|e| PorpoiseError::PtyError(format!("write: {e}")))?;
        Ok(())
    }
}

fn pty_resize_impl(fd: i32, rows: u16, cols: u16) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        let ws = nix::pty::Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
        let res = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws) };
        if res != 0 { Err(PorpoiseError::PtyError("resize failed".into())) } else { Ok(()) }
    }
    #[cfg(target_os = "windows")]
    {
        use portable_pty::PtySize;
        windows_pty::with_master(fd, |master| {
            let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
            let _ = master.resize(size);
        });
        Ok(())
    }
}

pub struct PtyManager {
    sessions: Arc<RwLock<HashMap<TerminalId, PtySession>>>,
    event_bus: EventBus,
    generation_id: Option<String>,
}

impl PtyManager {
    pub fn new(event_bus: EventBus) -> Self {
        Self { sessions: Arc::new(RwLock::new(HashMap::new())), event_bus, generation_id: None }
    }

    pub fn set_generation_id(&mut self, id: Option<String>) {
        self.generation_id = id;
    }

    pub async fn alloc(&self, rows: u16, cols: u16, shell: &str) -> Result<TerminalId> {
        let mut session = alloc_pty_impl(rows, cols, shell)?;
        session.generation_id.clone_from(&self.generation_id);
        let id = session.id;
        self.sessions.write().await.insert(id, session);
        self.spawn_read_loop(id);
        Ok(id)
    }

    /// Spawns a background tokio task that reads PTY output and publishes
    /// `TerminalEvent::Output` on the EventBus. The loop exits when the
    /// PTY session is closed (removed from sessions map).
    pub fn spawn_read_loop(&self, id: TerminalId) {
        let sessions = self.sessions.clone();
        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                let fd = {
                    let guard = sessions.read().await;
                    guard.get(&id).map(|s| s.fd)
                };
                let n = match fd {
                    Some(fd) => pty_read_impl(fd, &mut buf).await.ok().unwrap_or(0),
                    None => break,
                };
                if n > 0 {
                    let data = buf[..n].to_vec();
                    let timestamp = chrono::Utc::now().timestamp();
                    event_bus.publish(porpoise_core::types::event::SystemEvent::Terminal(
                        porpoise_core::types::event::TerminalEvent::Output { id, data, timestamp },
                    ));
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        });
    }

    pub async fn read(&self, id: TerminalId, buf: &mut [u8]) -> Result<usize> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&id).ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        pty_read_impl(session.fd, buf).await
    }

    pub async fn write(&self, id: TerminalId, data: &[u8]) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&id).ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        pty_write_impl(session.fd, data).await
    }

    pub async fn resize(&self, id: TerminalId, rows: u16, cols: u16) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&id).ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        pty_resize_impl(session.fd, rows, cols)
    }

    pub async fn close(&self, id: TerminalId) -> Result<()> {
        if let Some(session) = self.sessions.write().await.remove(&id) {
            #[cfg(target_os = "windows")]
            windows_pty::remove_master(session.fd);

            #[cfg(not(target_os = "windows"))]
            {
                // Send SIGHUP to child process to terminate gracefully
                if let Some(pid) = session.child_pid {
                    unsafe { libc::kill(pid as i32, libc::SIGHUP) };
                    // Reap child to prevent zombie process
                    unsafe { libc::waitpid(pid as i32, std::ptr::null_mut(), libc::WNOHANG) };
                }
                // Close the master file descriptor
                unsafe { libc::close(session.fd) };
            }
        }
        Ok(())
    }
}
