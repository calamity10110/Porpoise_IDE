use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use porpoise_core::error::{PorpoiseError, Result};
use porpoise_core::types::id::TerminalId;
use porpoise_core::bus::EventBus;

#[derive(Debug)]
pub struct PtySession {
    pub id: TerminalId,
    pub fd: i32,
    pub child_pid: u32,
    pub rows: u16,
    pub cols: u16,
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
    let (ptym, ptys) = pty::openpty(&winsize, None)
        .map_err(|e| PorpoiseError::PtyError(e.to_string()))?;

    let master_fd = ptym.as_raw_fd();
    let slave_fd = ptys.as_raw_fd();

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child }) => {
            unistd::close(slave_fd).ok();
            Ok(PtySession {
                id: TerminalId::new(),
                fd: master_fd,
                child_pid: child.as_raw(),
                rows,
                cols,
            })
        }
        Ok(ForkResult::Child) => {
            use std::os::unix::io::FromRawFd;
            unistd::setsid().ok();
            unistd::close(master_fd).ok();
            // Redirect stdin/stdout/stderr to slave
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
fn alloc_pty_impl(_rows: u16, _cols: u16, _shell: &str) -> Result<PtySession> {
    Err(PorpoiseError::Unimplemented("Windows PTY via ConPTY"))
}

async fn pty_read_impl(fd: i32, buf: &mut [u8]) -> Result<usize> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::io::FromRawFd;
        let std_file = unsafe { std::fs::File::from_raw_fd(fd) };
        let mut file = tokio::fs::File::from_std(std_file);
        let n = file.read(buf).await.map_err(|e| PorpoiseError::PtyError(e.to_string()))?;
        std::mem::forget(file);
        Ok(n)
    }
    #[cfg(target_os = "windows")]
    {
        let _ = (fd, buf);
        Err(PorpoiseError::Unimplemented("PTY read on Windows"))
    }
}

async fn pty_write_impl(fd: i32, data: &[u8]) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::io::FromRawFd;
        let std_file = unsafe { std::fs::File::from_raw_fd(fd) };
        let mut file = tokio::fs::File::from_std(std_file);
        file.write_all(data).await.map_err(|e| PorpoiseError::PtyError(e.to_string()))?;
        std::mem::forget(file);
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let _ = (fd, data);
        Err(PorpoiseError::Unimplemented("PTY write on Windows"))
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
        let _ = (fd, rows, cols);
        Err(PorpoiseError::Unimplemented("PTY resize on Windows"))
    }
}

pub struct PtyManager {
    sessions: Arc<RwLock<HashMap<TerminalId, PtySession>>>,
    _event_bus: EventBus,
}

impl PtyManager {
    pub fn new(event_bus: EventBus) -> Self {
        Self { sessions: Arc::new(RwLock::new(HashMap::new())), _event_bus: event_bus }
    }

    pub async fn alloc(&self, rows: u16, cols: u16, shell: &str) -> Result<TerminalId> {
        let session = alloc_pty_impl(rows, cols, shell)?;
        let id = session.id;
        self.sessions.write().await.insert(id, session);
        Ok(id)
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
        self.sessions.write().await.remove(&id);
        Ok(())
    }
}
