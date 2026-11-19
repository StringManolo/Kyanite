use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use nix::errno::Errno;
use nix::unistd::{fork, ForkResult};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() {
    println!("[init] Kyanite PID 1 initialized successfully.");

    let shell_pid = match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            let err = Command::new("/bin/brush").exec();
            eprintln!("[init] Failed to exec /bin/brush: {}", err);
            std::process::exit(1);
        }
        Ok(ForkResult::Parent { child }) => {
            println!("[init] Spawned login shell as PID {}", child);
            child
        }
        Err(e) => {
            eprintln!("[init] fork failed: {}", e);
            std::process::exit(1);
        }
    };

    println!("[init] Starting process reaper loop...");
    loop {
        reap_children();

        // Detect shell termination → poweroff (exit init → proot exits)
        match waitpid(shell_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {}
            Ok(_) | Err(Errno::ECHILD) => {
                println!("[init] Shell exited, powering off.");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("[init] waitpid error: {}", e);
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn reap_children() {
    loop {
        match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => break,
            Ok(WaitStatus::Exited(pid, status)) => {
                println!("[init] Child (PID {}) exited with status {}", pid, status);
            }
            Ok(WaitStatus::Signaled(pid, signal, _)) => {
                println!("[init] Child (PID {}) killed by signal {:?}", pid, signal);
            }
            Err(Errno::ECHILD) => break,
            Err(_) => break,
            _ => {}
        }
    }
}
