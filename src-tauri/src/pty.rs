use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Emitter;

pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
}

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(
        &self,
        id: String,
        rows: u16,
        cols: u16,
        cwd: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

        let mut cmd = CommandBuilder::new(&shell);
        cmd.arg("-l");

        if let Some(dir) = &cwd {
            cmd.cwd(dir);
        } else if let Some(home) = dirs::home_dir() {
            cmd.cwd(home);
        }

        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("TERM_PROGRAM", "FluxTerminal");
        cmd.env("LANG", "en_US.UTF-8");

        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let session_id = id.clone();
        let handle = app_handle.clone();

        // Reader thread — reads from PTY and sends to frontend
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let event_name = format!("pty-output-{}", session_id);
            let exit_event = format!("pty-exit-{}", session_id);

            eprintln!("[PTY] Reader thread started for {}", session_id);

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        eprintln!("[PTY] EOF for {}", session_id);
                        let _ = handle.emit(&exit_event, "closed");
                        break;
                    }
                    Ok(n) => {
                        // Convert bytes to String using lossy UTF-8 conversion
                        // This handles both text and ANSI escape sequences
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        eprintln!("[PTY] Read {} bytes from {}", n, session_id);

                        match handle.emit(&event_name, &text) {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("[PTY] Emit error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[PTY] Read error for {}: {}", session_id, e);
                        let _ = handle.emit(&exit_event, "error");
                        break;
                    }
                }
            }

            eprintln!("[PTY] Reader thread ended for {}", session_id);
        });

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get writer: {}", e))?;

        let session = PtySession {
            master: pair.master,
            writer,
        };

        self.sessions
            .lock()
            .map_err(|_| "Lock error".to_string())?
            .insert(id.clone(), session);

        eprintln!("[PTY] Session created: {}", id);

        Ok(())
    }

    pub fn write(&self, id: &str, data: &[u8]) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Lock error".to_string())?;
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| format!("Session not found: {}", id))?;

        eprintln!("[PTY] Writing {} bytes to {}", data.len(), id);

        session
            .writer
            .write_all(data)
            .map_err(|e| format!("Write error: {}", e))?;

        session
            .writer
            .flush()
            .map_err(|e| format!("Flush error: {}", e))?;

        Ok(())
    }

    pub fn resize(&self, id: &str, rows: u16, cols: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|_| "Lock error".to_string())?;
        let session = sessions
            .get(id)
            .ok_or_else(|| format!("Session not found: {}", id))?;

        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize error: {}", e))?;

        Ok(())
    }

    pub fn close(&self, id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Lock error".to_string())?;
        sessions.remove(id);
        eprintln!("[PTY] Session closed: {}", id);
        Ok(())
    }
}