use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use nix::errno::Errno;
use std::thread;
use std::time::Duration;

fn main() {
    println!("[init] Kyanite PID 1 initialized successfully.");
    println!("[init] Starting process reaper loop...");

    // Main loop for PID 1: Reap zombie processes continuously
    loop {
        reap_children();
        
        // Sleep briefly to prevent 100% CPU usage on busy-waiting
        thread::sleep(Duration::from_millis(50));
    }
}

/// Reaps all terminated zombie child processes
pub fn reap_children() {
    loop {
        // Wait for any child process without blocking (WNOHANG)
        match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {
                // No more exited children to reap at this moment
                break;
            }
            Ok(WaitStatus::Exited(pid, status)) => {
                println!("[init] Child process (PID {}) exited with status {}", pid, status);
            }
            Ok(WaitStatus::Signaled(pid, signal, core_dumped)) => {
                println!("[init] Child process (PID {}) killed by signal {:?} (core dumped: {})", pid, signal, core_dumped);
            }
            Err(Errno::ECHILD) => {
                // ECHILD means there are no child processes left to wait for
                break;
            }
            Err(e) => {
                eprintln!("[init] Warning: waitpid failed: {}", e);
                break;
            }
            _ => {}
        }
    }
}
