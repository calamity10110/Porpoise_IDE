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
    use std::os::unix::io::AsRawFd;

    use nix::{
        pty::{self, Winsize},
        unistd::{self, ForkResult},
    };

    let winsize = Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let (ptym, ptys) =
        pty::openpty(&winsize, None).map_err(|e: std::io::Error| PorpoiseError::PtyError(e.to_string()))?;

    let master_fd = ptym.as_raw_fd();
    let slave_fd = ptys.as_raw_fd();

    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child }) => {
            unistd::close(slave_fd).ok();
            Ok(PtySession {
                id: TerminalId::new(),
                fd: master_fd,
                child_pid: child.as_raw() as u32,
                rows,
                cols,
                generation_id: None,
            })
        }
        Ok(ForkResult::Child) => {
            unistd::setsid().ok();
            unistd::close(master_fd).ok();
            unsafe {
                libc::dup2(slave_fd, 0);
                libc::dup2(slave_fd, 1);
                libc::dup2(slave_fd, 2);
            }
            if slave_fd > 2 {
                unistd::close(slave_fd).ok();
            }
            let c_shell = std::ffi::CString::new(shell).unwrap();
            unistd::execvp(&c_shell, &[&c_shell]).ok();
            std::process::exit(1);
        }
        Err(e) => Err(PorpoiseError::PtyError(e.to_string())),
    }
}

#[cfg(target_os = "windows")]
mod windows_pty {
    use std::{
        collections::HashMap,
        io::{Read, Write},
        sync::{Arc, LazyLock, Mutex},
    };

    use portable_pty::MasterPty;

    // Each session's master/reader/writer live behind per-session mutexes,
    // cloned out of the map as Arcs. The global map lock is held only for
    // lookup — never across blocking I/O. Holding it during read() starved
    // writes of the lock and keystrokes hung forever.
    #[derive(Clone)]
    struct PtyHandle {
        master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
        reader: Arc<Mutex<Box<dyn Read + Send>>>,
        writer: Arc<Mutex<Box<dyn Write + Send>>>,
    }

    static PTYS: LazyLock<Mutex<HashMap<i32, PtyHandle>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

    fn lookup(fd: i32) -> std::result::Result<PtyHandle, String> {
        PTYS.lock()
            .map_err(|e| format!("pty lock poisoned: {e}"))?
            .get(&fd)
            .cloned()
            .ok_or_else(|| format!("master pty fd {fd} not found"))
    }

    pub fn store_master(fd: i32, master: Box<dyn MasterPty + Send>) -> std::result::Result<(), String> {
        let reader = master.try_clone_reader().map_err(|e| format!("clone reader: {e}"))?;
        // portable-pty's take_writer() is one-shot (Option::take): taken
        // exactly once here and reused for the session lifetime.
        let writer = master.take_writer().map_err(|e| format!("take writer: {e}"))?;
        PTYS.lock().map_err(|e| format!("pty lock poisoned: {e}"))?.insert(
            fd,
            PtyHandle {
                master: Arc::new(Mutex::new(master)),
                reader: Arc::new(Mutex::new(reader)),
                writer: Arc::new(Mutex::new(writer)),
            },
        );
        Ok(())
    }

    pub fn remove_master(fd: i32) {
        let _ = PTYS.lock().map(|mut m| {
            m.remove(&fd);
        });
    }

    pub fn read(fd: i32, buf: &mut [u8]) -> std::result::Result<usize, String> {
        let handle = lookup(fd)?;
        let mut reader = handle.reader.lock().map_err(|e| format!("reader lock poisoned: {e}"))?;
        reader.read(buf).map_err(|e| format!("read: {e}"))
    }

    pub fn write_all(fd: i32, data: &[u8]) -> std::result::Result<(), String> {
        let handle = lookup(fd)?;
        let mut writer = handle.writer.lock().map_err(|e| format!("writer lock poisoned: {e}"))?;
        writer.write_all(data).map_err(|e| format!("write: {e}"))
    }

    pub fn resize(fd: i32, rows: u16, cols: u16) -> std::result::Result<(), String> {
        let handle = lookup(fd)?;
        let master = handle.master.lock().map_err(|e| format!("master lock poisoned: {e}"))?;
        use portable_pty::PtySize;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("resize: {e}"))
    }
}

#[cfg(target_os = "windows")]
fn alloc_pty_impl(rows: u16, cols: u16, shell: &str) -> Result<PtySession> {
    use std::sync::atomic::{AtomicI32, Ordering};

    use portable_pty::{CommandBuilder, PtySize, native_pty_system};

    static NEXT_FD: AtomicI32 = AtomicI32::new(1000);

    let pty_system = native_pty_system();
    let size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = pty_system
        .openpty(size)
        .map_err(|e| PorpoiseError::PtyError(format!("openpty: {e}")))?;

    let cmd = CommandBuilder::new(shell);
    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| PorpoiseError::PtyError(format!("spawn: {e}")))?;

    let pid = child.process_id().unwrap_or(0);
    let fd = NEXT_FD.fetch_add(1, Ordering::Relaxed);

    windows_pty::store_master(fd, pair.master).map_err(PorpoiseError::PtyError)?;

    Ok(PtySession {
        id: TerminalId::new(),
        fd,
        child_pid: pid,
        rows,
        cols,
        generation_id: None,
    })
}

async fn pty_read_impl(fd: i32, buf: &mut [u8]) -> Result<usize> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::io::FromRawFd;

        use tokio::io::AsyncReadExt;
        // SAFETY: dup() duplicates the file descriptor. `fd` is a valid PTY master
        // fd stored in the session map. We check for errors (< 0) immediately.
        let dup_fd = unsafe { libc::dup(fd) };
        if dup_fd < 0 {
            return Err(PorpoiseError::PtyError("dup failed".into()));
        }
        // SAFETY: from_raw_fd takes ownership of the fd. `dup_fd` was just created
        // by dup() above and is guaranteed valid (>= 0). The File destructor will close it.
        let std_file = unsafe { std::fs::File::from_raw_fd(dup_fd) };
        let mut file = tokio::fs::File::from_std(std_file);
        file.read(buf).await.map_err(|e| PorpoiseError::PtyError(e.to_string()))
    }
    #[cfg(target_os = "windows")]
    {
        let n_buf = buf.len().min(4096);
        let (n, data) = tokio::task::spawn_blocking(move || -> std::result::Result<(usize, Vec<u8>), String> {
            let mut own = vec![0u8; n_buf];
            let n = windows_pty::read(fd, &mut own)?;
            Ok((n, own))
        })
        .await
        .map_err(|e| PorpoiseError::PtyError(format!("blocking read: {e}")))?
        .map_err(PorpoiseError::PtyError)?;
        buf[..n].copy_from_slice(&data[..n]);
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
        // SAFETY: from_raw_fd takes ownership of the fd. `dup_fd` was just created
        // by dup() above and is guaranteed valid (>= 0). The File destructor will close it.
        let std_file = unsafe { std::fs::File::from_raw_fd(dup_fd) };
        let mut file = tokio::fs::File::from_std(std_file);
        file.write_all(data)
            .await
            .map_err(|e| PorpoiseError::PtyError(e.to_string()))
    }
    #[cfg(target_os = "windows")]
    {
        let data = data.to_vec();
        tokio::task::spawn_blocking(move || windows_pty::write_all(fd, &data))
            .await
            .map_err(|e| PorpoiseError::PtyError(format!("blocking: {e}")))?
            .map_err(PorpoiseError::PtyError)?;
        Ok(())
    }
}

fn pty_resize_impl(fd: i32, rows: u16, cols: u16) -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        let ws = nix::pty::Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: ioctl with TIOCSWINSZ sets the terminal window size. `fd` is a
        // valid PTY master fd. `ws` is a valid stack-allocated Winsize struct.
        let res = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws) };
        if res != 0 {
            Err(PorpoiseError::PtyError("resize failed".into()))
        } else {
            Ok(())
        }
    }
    #[cfg(target_os = "windows")]
    {
        windows_pty::resize(fd, rows, cols).map_err(PorpoiseError::PtyError)?;
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
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            generation_id: None,
        }
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
        let session = sessions
            .get(&id)
            .ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        pty_read_impl(session.fd, buf).await
    }

    pub async fn write(&self, id: TerminalId, data: &[u8]) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&id)
            .ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
        pty_write_impl(session.fd, data).await
    }

    pub async fn resize(&self, id: TerminalId, rows: u16, cols: u16) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&id)
            .ok_or_else(|| PorpoiseError::Terminal("session not found".into()))?;
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
                    // SAFETY: kill() sends SIGHUP to the child process to trigger
                    // graceful shutdown. `pid` comes from session.child_pid which was
                    // captured at spawn time. Signal delivery to a non-existent PID is
                    // a harmless no-op (ESRCH).
                    unsafe { libc::kill(pid as i32, libc::SIGHUP) };
                    // Reap child to prevent zombie process
                    // SAFETY: waitpid with WNOHANG reaps the child without blocking.
                    // `pid` is the known child PID. null_mut() discards status info.
                    // WNOHANG ensures we never block the async runtime.
                    unsafe { libc::waitpid(pid as i32, std::ptr::null_mut(), libc::WNOHANG) };
                }
                // Close the master file descriptor
                // SAFETY: close() releases the PTY master file descriptor.
                // `session.fd` was allocated by openpty and is being removed from
                // the session map, so no other code will use it after this point.
                unsafe { libc::close(session.fd) };
            }
        }
        Ok(())
    }
}
