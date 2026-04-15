use std::path::PathBuf;
use std::process::{Child, Command};

/// Records audio using sox `rec` command — same approach as Hammerspoon prototype.
pub struct AudioRecorder {
    process: Option<Child>,
    output_path: PathBuf,
}

impl AudioRecorder {
    pub fn start(output_path: PathBuf) -> Result<Self, String> {
        let rec_path = find_rec()?;

        // Remove old file to avoid appending
        let _ = std::fs::remove_file(&output_path);

        let child = Command::new(&rec_path)
            .args(["-c", "1", "-r", "16000", "-b", "16"])
            .arg(output_path.as_os_str())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start rec: {}", e))?;

        Ok(Self {
            process: Some(child),
            output_path,
        })
    }

    pub fn stop(mut self) -> Result<PathBuf, String> {
        if let Some(ref mut child) = self.process {
            let pid = child.id() as i32;

            // Send SIGTERM
            #[cfg(unix)]
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }

            // Wait with timeout (max 2 seconds)
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if start.elapsed().as_secs() > 2 {
                            // Force kill if stuck
                            let _ = child.kill();
                            let _ = child.wait();
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }

            self.process = None;
        }

        // Brief pause for file to be flushed
        std::thread::sleep(std::time::Duration::from_millis(100));

        if !self.output_path.exists() {
            return Err("Recording file was not created".to_string());
        }

        let metadata = std::fs::metadata(&self.output_path)
            .map_err(|e| format!("Cannot read recording file: {}", e))?;

        if metadata.len() < 1000 {
            return Err("Recording too short or empty".to_string());
        }

        Ok(self.output_path.clone())
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.process {
            #[cfg(unix)]
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
            let _ = child.wait();
        }
    }
}

fn find_rec() -> Result<String, String> {
    for path in &[
        "/opt/homebrew/bin/rec",
        "/usr/local/bin/rec",
        "/usr/bin/rec",
    ] {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    Err("sox (rec) not found. Install with: brew install sox".to_string())
}
