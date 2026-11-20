
use nix::errno::Errno;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};

use std::io::{self, IsTerminal, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const SHELL_PATH: &str = "/bin/brush";

const TEST_TIMEOUT: Duration = Duration::from_secs(5);
const TEST_POLL_INTERVAL: Duration = Duration::from_millis(100);

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const LOOP_INTERVAL: Duration = Duration::from_millis(100);

// ============================================================
// LOGGING
// ============================================================

fn log(message: &str) {
    eprintln!("[init] {}", message);
    let _ = io::stderr().flush();
}

fn log_stdio(context: &str) {
    log(&format!(
        "{}: stdin_tty={}, stdout_tty={}, stderr_tty={}",
        context,
        io::stdin().is_terminal(),
        io::stdout().is_terminal(),
        io::stderr().is_terminal()
    ));
}

fn log_environment() {
    log("========== ENVIRONMENT ==========");

    for name in [
        "HOME",
        "USER",
        "LOGNAME",
        "SHELL",
        "TERM",
        "COLORTERM",
        "LANG",
        "PATH",
        "PWD",
        "PROOT_TMP_DIR",
    ] {
        match std::env::var(name) {
            Ok(value) => log(&format!("ENV {}={}", name, value)),
            Err(_) => log(&format!("ENV {}=<not set>", name)),
        }
    }

    match std::env::current_dir() {
        Ok(path) => log(&format!("Current directory: {}", path.display())),
        Err(error) => log(&format!("Cannot get current directory: {}", error)),
    }

    log_stdio("Init process");

    log("================================");
}

// ============================================================
// TEST 1: NON-INTERACTIVE BRUSH
// ============================================================

fn run_noninteractive_test() {
    log("");
    log("========================================");
    log(" TEST 1: NON-INTERACTIVE BRUSH");
    log("========================================");

    log("Preparing non-interactive Brush test.");

    if !std::path::Path::new(SHELL_PATH).exists() {
        log("ERROR: /bin/brush does not exist.");
        log("Skipping non-interactive test.");
        return;
    }

    // Commands are executed by Brush itself.
    //
    // We deliberately do not require an interactive terminal.
    // stdin is disconnected to verify non-interactive behavior.

    let script = r#"
echo "========== BRUSH NON-INTERACTIVE TEST =========="
echo "Brush command execution: OK"
echo ""
echo "Current directory:"
pwd
echo ""
echo "Root filesystem listing:"

if command -v ls >/dev/null 2>&1; then
    ls -la /
else
    echo "WARNING: ls is not installed."
fi

echo ""
echo "Testing glibc env executable:"
/usr/bin/env

echo ""
echo "========== NON-INTERACTIVE TEST FINISHED =========="
"#;

    log("Command: /bin/brush -c <test script>");
    log("Interactive mode: DISABLED");
    log("stdin: /dev/null");
    log("stdout: inherited");
    log("stderr: inherited");

    let mut child = match Command::new(SHELL_PATH)
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            log(&format!(
                "NON-INTERACTIVE: Brush spawned successfully. PID={}",
                child.id()
            ));

            child
        }

        Err(error) => {
            log("NON-INTERACTIVE: FAILED TO SPAWN BRUSH.");
            log(&format!("Error: {}", error));
            log(&format!("Error kind: {:?}", error.kind()));
            log(&format!("OS error: {:?}", error.raw_os_error()));

            return;
        }
    };

    log("Waiting for non-interactive Brush to finish...");

    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                log("NON-INTERACTIVE: Brush exited naturally.");

                log(&format!("Exit status: {}", status));

                if let Some(code) = status.code() {
                    log(&format!("Exit code: {}", code));
                }

                if let Some(signal) = status.signal() {
                    log(&format!("Terminating signal: {}", signal));
                }

                log(&format!(
                    "Execution duration: {:?}",
                    start.elapsed()
                ));

                break;
            }

            Ok(None) => {
                if start.elapsed() >= TEST_TIMEOUT {
                    log("NON-INTERACTIVE: TIMEOUT REACHED.");
                    log("Brush is still running after 5 seconds.");
                    log("Sending SIGKILL to diagnostic Brush process...");

                    match child.kill() {
                        Ok(()) => {
                            log("SIGKILL sent successfully.");
                        }

                        Err(error) => {
                            log(&format!("ERROR sending SIGKILL: {}", error));
                        }
                    }

                    log("Waiting for diagnostic Brush process to be reaped...");

                    match child.wait() {
                        Ok(status) => {
                            log(&format!(
                                "Diagnostic Brush reaped. Status: {}",
                                status
                            ));
                        }

                        Err(error) => {
                            log(&format!(
                                "ERROR waiting for diagnostic Brush: {}",
                                error
                            ));
                        }
                    }

                    break;
                }

                thread::sleep(TEST_POLL_INTERVAL);
            }

            Err(error) => {
                log("NON-INTERACTIVE: ERROR checking child status.");
                log(&format!("Error: {}", error));

                log("Attempting to terminate diagnostic process...");

                let _ = child.kill();

                match child.wait() {
                    Ok(status) => {
                        log(&format!("Child reaped: {}", status));
                    }

                    Err(wait_error) => {
                        log(&format!("Wait error: {}", wait_error));
                    }
                }

                break;
            }
        }
    }

    log("NON-INTERACTIVE TEST FINISHED.");
    log("Kyanite init is still running.");
    log("Proceeding to interactive test.");

    log("========================================");
    log("");
}

// ============================================================
// TEST 2: INTERACTIVE BRUSH
// ============================================================

fn start_interactive_shell() -> Result<Pid, Errno> {
    log("========================================");
    log(" TEST 2: INTERACTIVE BRUSH");
    log("========================================");

    log("Checking Brush executable...");

    if !std::path::Path::new(SHELL_PATH).exists() {
        log("FATAL: /bin/brush does not exist.");
        std::process::exit(127);
    }

    log("Brush executable exists.");

    log_stdio("Before fork");

    log("Preparing fork()...");

    let _ = io::stdout().flush();
    let _ = io::stderr().flush();

    match unsafe { fork()? } {
        ForkResult::Child => {
            log("========== INTERACTIVE CHILD ==========");

            log(&format!("Child PID: {}", std::process::id()));

            log_stdio("Interactive child");

            log("Arguments: /bin/brush -i -l");
            log("Interactive mode: FORCED");
            log("Login mode: ENABLED");

            log("stdin: inherited");
            log("stdout: inherited");
            log("stderr: inherited");

            log("Executing Brush now...");

            let error = Command::new(SHELL_PATH)
                .arg("-i")
                .arg("-l")
                .exec();

            // Only reached if exec fails.

            log("========== INTERACTIVE EXEC FAILED ==========");

            log(&format!("Error: {}", error));
            log(&format!("Error kind: {:?}", error.kind()));
            log(&format!("OS error: {:?}", error.raw_os_error()));

            log("=============================================");

            std::process::exit(127);
        }

        ForkResult::Parent { child } => {
            log(&format!(
                "Interactive Brush spawned. PID={}",
                child
            ));

            log("Interactive shell should now be available.");

            log("========================================");

            Ok(child)
        }
    }
}

// ============================================================
// CHILD REAPER
// ============================================================

fn reap_other_children() {
    loop {
        match waitpid(None, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => break,

            Ok(WaitStatus::Exited(pid, status)) => {
                log(&format!(
                    "REAPER: Child PID {} exited with status {}",
                    pid, status
                ));
            }

            Ok(WaitStatus::Signaled(pid, signal, core_dumped)) => {
                log(&format!(
                    "REAPER: Child PID {} killed by {:?}, core_dumped={}",
                    pid, signal, core_dumped
                ));
            }

            Ok(WaitStatus::Stopped(pid, signal)) => {
                log(&format!(
                    "REAPER: Child PID {} stopped by {:?}",
                    pid, signal
                ));
                break;
            }

            Ok(WaitStatus::Continued(pid)) => {
                log(&format!(
                    "REAPER: Child PID {} continued",
                    pid
                ));
            }

            Ok(_) => break,

            Err(Errno::ECHILD) => break,

            Err(Errno::EINTR) => continue,

            Err(error) => {
                log(&format!("REAPER ERROR: {:?}", error));
                break;
            }
        }
    }
}

// ============================================================
// INTERACTIVE SHELL SUPERVISOR
// ============================================================

fn monitor_shell(shell_pid: Pid) {
    log("========================================");
    log(" STARTING PROCESS SUPERVISOR");
    log("========================================");

    log(&format!("Monitoring Brush PID: {}", shell_pid));

    log_stdio("Supervisor");

    let mut last_heartbeat = Instant::now();

    loop {
        // Check the main shell FIRST.
        // This prevents the generic reaper from consuming
        // its exit status before we inspect it.

        match waitpid(shell_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => {}

            Ok(WaitStatus::Exited(pid, status)) => {
                log(&format!(
                    "SHELL EXITED: PID={}, status={}",
                    pid, status
                ));

                log("Kyanite session is shutting down.");

                std::process::exit(status);
            }

            Ok(WaitStatus::Signaled(pid, signal, core_dumped)) => {
                log(&format!(
                    "SHELL TERMINATED: PID={}, signal={:?}, core_dumped={}",
                    pid, signal, core_dumped
                ));

                log("Kyanite session is shutting down.");

                std::process::exit(128);
            }

            Ok(other) => {
                log(&format!("SHELL WAIT STATUS: {:?}", other));
            }

            Err(Errno::ECHILD) => {
                log("ERROR: Main shell was already reaped.");

                log("Kyanite session is shutting down.");

                std::process::exit(127);
            }

            Err(Errno::EINTR) => {
                continue;
            }

            Err(error) => {
                log(&format!("waitpid() error: {:?}", error));
            }
        }

        // Reap any other children without losing the main
        // shell's exit status.

        reap_other_children();

        // Periodic diagnostics.

        if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
            log(&format!(
                "HEARTBEAT: Interactive Brush PID {} is alive.",
                shell_pid
            ));

            log_stdio("Supervisor");

            last_heartbeat = Instant::now();
        }

        thread::sleep(LOOP_INTERVAL);
    }
}

// ============================================================
// MAIN
// ============================================================

fn main() {
    log("");
    log("========================================");
    log("          KYANITE INIT");
    log("========================================");

    log(&format!("Init PID: {}", std::process::id()));

    log("Stage 1: Inspecting environment.");

    log_environment();

    // First test: non-interactive Brush.
    //
    // This test runs in a separate child process.
    // Its termination does NOT terminate Kyanite.

    run_noninteractive_test();

    // Second test: interactive Brush.

    log("Stage 2: Starting interactive Brush.");

    let shell_pid = match start_interactive_shell() {
        Ok(pid) => pid,

        Err(error) => {
            log(&format!("FATAL: fork() failed: {:?}", error));

            std::process::exit(1);
        }
    };

    log("Stage 3: Interactive shell spawned.");

    log(&format!("Shell PID: {}", shell_pid));

    log("Stage 4: Starting supervisor.");

    monitor_shell(shell_pid);
}

// Unix ExitStatusExt for signal diagnostics.
use std::os::unix::process::ExitStatusExt;

