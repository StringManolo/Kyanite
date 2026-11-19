use std::process;
use std::thread;
use std::time::Duration;
use nix::sys::signal::{sigaction, SigAction, SigFlag, SigSet, SaFlags};
use nix::unistd::pid_t;

extern "C" fn handle_sigchld(_sig: i32) {
    // Bucle para recolectar todos los procesos zombie hijos huérfanos
    loop {
        match nix::sys::wait::waitpid(
            nix::unistd::Pid::from_raw(-1),
            Some(nix::sys::wait::WaitPidFlag::WNOHANG),
        ) {
            Ok(res) => match res {
                nix::sys::wait::WaitStatus::StillAlive => break,
                nix::sys::wait::WaitStatus::Exited(pid, _) => {
                    // Proceso hijo terminado normalmente
                    let _ = pid;
                }
                nix::sys::wait::WaitStatus::Signaled(pid, sig, _) => {
                    // Proceso hijo terminado por una señal
                    let _ = (pid, sig);
                }
                _ => {}
            },
            Err(_) => break, // No quedan hijos por recolectar
        }
    }
}

fn main() {
    println!("Kyanite Init (PID 1) started in PRoot enviroment.");

    // 1. Configurar el manejo de señales para recolectar zombies (SIGCHLD)
    unsafe {
        let sa = SigAction::new(
            nix::sys::signal::SigHandler::Handler(handle_sigchld),
            SaFlags::SA_RESTART,
            SigSet::empty(),
        );
        let _ = sigaction(nix::sys::signal::Signal::SIGCHLD, &sa);
    }

    // 2. Bucle principal del PID 1
    // En PRoot, el init se mantiene a la escucha y recolectando procesos en segundo plano.
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
