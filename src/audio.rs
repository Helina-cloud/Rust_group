use std::process::{Command, Stdio};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

pub struct AudioSystem {
    stop_flag: Arc<AtomicBool>,
    bgm_thread: Option<thread::JoinHandle<()>>,
}

impl AudioSystem {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
            bgm_thread: None,
        }
    }

    pub fn play_bgm(&mut self, path: &str) {
        self.stop_bgm(); // 先停止当前播放
        
        let stop_flag = self.stop_flag.clone();
        let path = path.to_string();
        
        self.stop_flag.store(false, Ordering::Relaxed);
        
        self.bgm_thread = Some(thread::spawn(move || {
            while !stop_flag.load(Ordering::Relaxed) {
                let status = if cfg!(target_os = "windows") {
                    Command::new("powershell")
                        .args(&[
                            "-c",
                            &format!("(New-Object Media.SoundPlayer '{}').PlaySync()", path)
                        ])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                } else if cfg!(target_os = "macos") {
                    Command::new("afplay")
                        .arg(&path)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                } else {
                    Command::new("mpg123")
                        .args(&["-q", "--loop", "-1", &path])
                        .status()
                };
                
                if let Ok(exit_status) = status {
                    if !exit_status.success() {
                        break;
                    }
                }
                
                // 短暂延迟防止CPU占用过高
                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    pub fn stop_bgm(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(thread) = self.bgm_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for AudioSystem {
    fn drop(&mut self) {
        self.stop_bgm();
    }
}