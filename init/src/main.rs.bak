use nix::errno::Errno;
use nix::libc;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};

use std::ffi::OsString;
use std::io::{self, IsTerminal, Read, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const SHELL_PATH: &str = "/bin/brush";

// ============================================================
// CONFIGURACION
// ============================================================

/// true  = al arrancar pregunta que modo de entrada de Brush usar.
/// false = arranca directo con DEFAULT_MODE (para la version final).
const BOOT_MENU: bool = false;

/// Indice (0-based) en MODES que se usa con Enter / sin menu. 0 = reedline.
const DEFAULT_MODE: usize = 0;

/// Latidos de diagnostico (uno cada 5 s) tras lanzar el shell. 0 = ninguno.
const HEARTBEATS_MAX: u32 = 0;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const LOOP_INTERVAL: Duration = Duration::from_millis(100);
const TEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Si el shell falla despues de este tiempo, init se cierra en vez de volver al menu.
const QUICK_EXIT: Duration = Duration::from_secs(60);
const MAX_ATTEMPTS: u32 = 10;

/// Ticks de CPU (1 tick = 10 ms) en un latido a partir de los cuales se considera bucle activo.
const BUSY_TICKS: u64 = 50;

/// true = banner, entorno, version de brush, TEST 1 y trazas de arranque.
const DEBUG_BOOT: bool = false;

struct Mode {
    name: &'static str,
    /// Texto que debe aparecer en `brush --help` para que el modo este soportado.
    probe: Option<&'static str>,
    args: &'static [&'static str],
}

const MODES: &[Mode] = &[
    Mode { name: "reedline (por defecto de brush)", probe: None, args: &["-i", "-l"] },
    Mode { name: "basic", probe: Some("input-backend"), args: &["--input-backend", "basic", "-i", "-l"] },
    Mode { name: "minimal", probe: Some("input-backend"), args: &["--input-backend", "minimal", "-i", "-l"] },
    Mode { name: "noediting", probe: Some("noediting"), args: &["--noediting", "-i", "-l"] },
];

// ============================================================
// LOGGING
// ============================================================
// Se escribe "\r\n" a mano: cuando el shell deja el tty en modo raw
// (OPOST desactivado) un "\n" suelto no vuelve a la columna 0 y los
// logs salen en escalera.

fn log(message: &str) {
    let mut err = io::stderr().lock();
    let _ = write!(err, "[init] {}\r\n", message);
    let _ = err.flush();
}

fn debug(message: &str) {
    if DEBUG_BOOT {
        log(message);
    }
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
        "HOME", "USER", "SHELL", "TERM", "COLORTERM", "LANG", "PATH", "PWD", "PROOT_TMP_DIR",
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
// INFO DE BRUSH (version y flags soportados)
// ============================================================

fn brush_output(arg: &str) -> String {
    match Command::new(SHELL_PATH).arg(arg).stdin(Stdio::null()).output() {
        Ok(out) => {
            let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            text
        }
        Err(error) => {
            log(&format!("no se pudo ejecutar brush {}: {}", arg, error));
            String::new()
        }
    }
}

fn log_brush_info() -> String {
    let version = brush_output("--version");
    log(&format!("brush --version: {}", version.trim()));

    let help = brush_output("--help");
    let mut found = false;
    for line in help.lines() {
        if line.contains("input-backend") || line.contains("noediting") {
            log(&format!("brush --help: {}", line.trim()));
            found = true;
        }
    }
    if !found {
        log("brush --help: no menciona input-backend ni noediting");
    }
    help
}

// ============================================================
// TEST 1: BRUSH NO INTERACTIVO
// ============================================================

fn run_noninteractive_test() {
    log("--- TEST 1: brush -c (no interactivo) ---");

    let script = "echo 'brush -c: OK'; pwd; /usr/bin/env >/dev/null && echo 'glibc env: OK'";

    let mut child = match Command::new(SHELL_PATH)
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            log(&format!("TEST 1: no se pudo lanzar brush: {} ({:?})", error, error.kind()));
            return;
        }
    };

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                log(&format!("TEST 1: brush termino: {} en {:?}", status, start.elapsed()));
                return;
            }
            Ok(None) => {
                if start.elapsed() >= TEST_TIMEOUT {
                    log("TEST 1: TIMEOUT, matando brush de prueba");
                    let _ = child.kill();
                    let _ = child.wait();
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                log(&format!("TEST 1: error en try_wait: {}", error));
                let _ = child.kill();
                let _ = child.wait();
                return;
            }
        }
    }
}

// ============================================================
// MENU DE ARRANQUE
// ============================================================

fn choose_mode(help: &str) -> usize {
    if !BOOT_MENU {
        return DEFAULT_MODE;
    }

    log("");
    log("Elige el modo de entrada de Brush (Enter = por defecto):");
    for (i, mode) in MODES.iter().enumerate() {
        let supported = match mode.probe {
            Some(probe) if !help.is_empty() => help.contains(probe),
            _ => true,
        };
        log(&format!(
            "  [{}] {}{}{}",
            i + 1,
            mode.name,
            if i == DEFAULT_MODE { "  (por defecto)" } else { "" },
            if supported { "" } else { "  <-- este brush NO parece soportarlo" }
        ));
    }
    log("  [q] salir");

    {
        let mut err = io::stderr().lock();
        let _ = write!(err, "[init] > ");
        let _ = err.flush();
    }

    // Se lee con un unico read(): funciona igual si el tty esta en modo
    // canonico (llega "2\n") que si un shell anterior lo dejo en raw
    // (llega "2"). Se usa el primer caracter no blanco.
    let mut buf = [0u8; 256];
    let choice = match io::stdin().read(&mut buf) {
        Ok(n) => buf[..n].iter().copied().find(|b| !b.is_ascii_whitespace()),
        Err(error) => {
            log(&format!("error leyendo stdin: {}", error));
            None
        }
    };

    match choice {
        Some(b'q') | Some(b'Q') => {
            log("saliendo");
            std::process::exit(0);
        }
        Some(c) if c >= b'1' && (c - b'1') < MODES.len() as u8 => {
            let index = (c - b'1') as usize;
            log(&format!("elegido: {}", MODES[index].name));
            index
        }
        _ => {
            log(&format!("usando el modo por defecto: {}", MODES[DEFAULT_MODE].name));
            DEFAULT_MODE
        }
    }
}

// ============================================================
// LANZAR EL SHELL INTERACTIVO
// ============================================================

// ------------------------------------------------------------
// JOB CONTROL / TTY
// ------------------------------------------------------------
// Brush se pone a si mismo en un grupo de procesos nuevo (pgrp = su PID).
// Si ese grupo no es el grupo en primer plano del tty, cada lectura del
// teclado le manda SIGTTIN. Por eso init hace de "login": crea el grupo
// y se lo entrega al shell como primer plano ANTES del exec.

fn ignore_sigttou() {
    // Necesario para poder llamar a tcsetpgrp() desde un grupo en segundo plano.
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
    }
}

/// Se ejecuta en el hijo, justo antes del exec.
fn child_job_control() {
    unsafe {
        let pid = libc::getpid();

        if libc::setpgid(0, 0) != 0 {
            log(&format!("setpgid(0, 0) FALLO: {}", io::Error::last_os_error()));
        }

        if libc::tcsetpgrp(0, pid) != 0 {
            log(&format!("tcsetpgrp(0, {}) FALLO: {}", pid, io::Error::last_os_error()));
        } else {
            debug(&format!("tcsetpgrp(0, {}) OK: el shell es el grupo en primer plano", pid));
        }

        // El shell recibe las senales de job control con la accion por defecto.
        libc::signal(libc::SIGTTOU, libc::SIG_DFL);
        libc::signal(libc::SIGTTIN, libc::SIG_DFL);
        libc::signal(libc::SIGTSTP, libc::SIG_DFL);
    }
}

/// Devuelve el tty a init cuando el shell termina (para poder leer el menu).
fn reclaim_tty() {
    unsafe {
        let pgrp = libc::getpgrp();
        if libc::tcsetpgrp(0, pgrp) != 0 {
            log(&format!("tcsetpgrp(0, {}) FALLO al recuperar el tty: {}", pgrp, io::Error::last_os_error()));
        }
    }
}

fn start_shell(args: &[&str]) -> Result<Pid, Errno> {
    let _ = io::stdout().flush();
    let _ = io::stderr().flush();

    match unsafe { fork()? } {
        ForkResult::Child => {
            child_job_control();

            let error = Command::new(SHELL_PATH)
                .args(args)
                .env("SHELL", SHELL_PATH)
                .exec();

            log("========== EXEC FALLO ==========");
            log(&format!("Error: {}", error));
            log(&format!("Error kind: {:?}, OS error: {:?}", error.kind(), error.raw_os_error()));
            std::process::exit(127);
        }
        ForkResult::Parent { child } => Ok(child),
    }
}

// ============================================================
// SUPERVISOR
// ============================================================

struct ProcInfo {
    state: char,
    pgrp: i64,
    /// Grupo de procesos en primer plano del tty del proceso.
    tpgid: i64,
    cpu_ticks: u64,
}

fn read_proc(pid: Pid) -> Option<ProcInfo> {
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid.as_raw())).ok()?;
    let rest = &stat[stat.rfind(')')? + 1..];
    let f: Vec<&str> = rest.split_whitespace().collect();
    // f[0]=estado f[2]=pgrp f[5]=tpgid f[11]=utime f[12]=stime
    Some(ProcInfo {
        state: f.first()?.chars().next()?,
        pgrp: f.get(2)?.parse().ok()?,
        tpgid: f.get(5)?.parse().ok()?,
        cpu_ticks: f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?,
    })
}

fn diagnose(info: &ProcInfo, delta: u64) -> &'static str {
    if info.state == 'T' {
        "PARADO por senal (T): job control (SIGTTIN/SIGTTOU/SIGTSTP)"
    } else if info.tpgid > 0 && info.pgrp != info.tpgid {
        "el shell NO esta en el grupo en primer plano del tty: leer del tty le manda SIGTTIN"
    } else if delta >= BUSY_TICKS {
        "el shell CONSUME CPU: bucle activo, no espera entrada (tipico de reedline/crossterm sin poder leer el terminal). Reinicia la sesion y prueba basic/minimal"
    } else {
        "el shell duerme esperando entrada o un evento (normal si ya ves el prompt)"
    }
}

fn heartbeat(shell: Pid, n: u32, last_cpu: &mut u64) {
    match read_proc(shell) {
        Some(info) => {
            let delta = info.cpu_ticks.saturating_sub(*last_cpu);
            *last_cpu = info.cpu_ticks;
            log(&format!(
                "HEARTBEAT #{} shell={} estado={} pgrp={} tty_fg_pgrp={} cpu_ticks={} (+{})",
                n, shell, info.state, info.pgrp, info.tpgid, info.cpu_ticks, delta
            ));
            log(&format!("  diagnostico: {}", diagnose(&info, delta)));
        }
        None => log(&format!("HEARTBEAT #{}: no puedo leer /proc/{}/stat", n, shell)),
    }
}

enum Outcome {
    Exited(i32),
    Signaled(String),
    Lost,
}

fn supervise(shell: Pid) -> Outcome {
    debug("--- SUPERVISOR ---");

    let mut last_beat = Instant::now();
    let mut beats = 0u32;
    let mut last_cpu = 0u64;

    loop {
        // Recoge eventos de cualquier hijo (shell y huerfanos adoptados).
        loop {
            match waitpid(None, Some(WaitPidFlag::WNOHANG | WaitPidFlag::WUNTRACED)) {
                Ok(WaitStatus::StillAlive) => break,

                Ok(WaitStatus::Exited(pid, code)) => {
                    if pid == shell {
                        return Outcome::Exited(code);
                    }
                    log(&format!("REAPER: PID {} salio con {}", pid, code));
                }

                Ok(WaitStatus::Signaled(pid, signal, _core)) => {
                    if pid == shell {
                        return Outcome::Signaled(format!("{:?}", signal));
                    }
                    log(&format!("REAPER: PID {} muerto por {:?}", pid, signal));
                }

                Ok(WaitStatus::Stopped(pid, signal)) => {
                    log(&format!(
                        "PARADO: PID {} por {:?}{}",
                        pid,
                        signal,
                        if pid == shell { "  <-- ES EL SHELL: job control" } else { "" }
                    ));
                }

                Ok(other) => log(&format!("WAIT: {:?}", other)),

                Err(Errno::ECHILD) => return Outcome::Lost,
                Err(Errno::EINTR) => continue,
                Err(error) => {
                    log(&format!("waitpid error: {:?}", error));
                    break;
                }
            }
        }

        if HEARTBEATS_MAX > 0 && beats < HEARTBEATS_MAX && last_beat.elapsed() >= HEARTBEAT_INTERVAL {
            beats += 1;
            heartbeat(shell, beats, &mut last_cpu);
            last_beat = Instant::now();
        }

        thread::sleep(LOOP_INTERVAL);
    }
}

// ============================================================
// MODO "sh" NO INTERACTIVO (paso directo a Brush)
// ============================================================
// /bin/sh es un enlace a este binario. Programas como git, make, python (subprocess), xargs o
// cron ejecutan `/bin/sh -c "<comando>"` o `/bin/sh script`, o le pasan los comandos por stdin.
// Eso NO debe abrir una sesion de login: debe ejecutar el comando y devolver su codigo de salida.
// Aqui se decide, a partir de los argumentos, si hay que hacer `exec` directo de Brush con los
// mismos argumentos. La sesion (grupo de procesos, tty, supervisor) queda para el arranque real
// (PID 1) y para el uso interactivo: `sh`, `sh -l`, `sh -il`.

/// true = no hay que montar sesion: exec directo de Brush con los argumentos recibidos.
fn wants_passthrough(args: &[String], is_pid1: bool, stdin_is_tty: bool) -> bool {
    if is_pid1 {
        return false;
    }
    let mut interactive = false;
    for a in args {
        if a == "--" || a == "-" {
            return true; // lo que sigue es script / argumentos; "-" = script por stdin
        }
        if a.starts_with("--") {
            if a == "--login" || a == "--interactive" {
                interactive = true;
            }
            continue;
        }
        if a.starts_with('-') && a.len() > 1 {
            let flags = &a[1..];
            if flags.contains('c') || flags.contains('s') {
                return true; // -c <comando>  /  -s (leer comandos de stdin)
            }
            if flags.contains('i') || flags.contains('l') {
                interactive = true;
            }
            continue;
        }
        return true; // primer argumento que no es opcion: ruta de un script
    }
    if interactive {
        return false;
    }
    !stdin_is_tty // `echo cmd | sh`
}

/// Sustituye este proceso por Brush con los mismos argumentos. No vuelve.
/// Debe llamarse ANTES de ignore_sigttou(): una senal ignorada se hereda tras exec.
fn exec_passthrough(args: &[String]) -> ! {
    let error = Command::new(SHELL_PATH).args(args).exec();
    eprintln!("sh: no se pudo ejecutar {}: {}", SHELL_PATH, error);
    std::process::exit(127);
}

// ============================================================
// MAIN
// ============================================================

fn main() {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a: OsString| a.to_string_lossy().into_owned())
        .collect();
    if wants_passthrough(&args, std::process::id() == 1, io::stdin().is_terminal()) {
        exec_passthrough(&args);
    }

    debug("");
    debug("========================================");
    debug("          KYANITE INIT");
    debug("========================================");
    debug(&format!("Init PID: {}", std::process::id()));
    ignore_sigttou();

    if DEBUG_BOOT {
        log_environment();
    }
    let help = if DEBUG_BOOT { log_brush_info() } else { String::new() };
    if DEBUG_BOOT {
        run_noninteractive_test();
    }

    let mut attempts = 0u32;
    loop {
        attempts += 1;

        let index = choose_mode(&help);
        let mode = &MODES[index];
        debug(&format!("lanzando [{}]: {} {}", mode.name, SHELL_PATH, mode.args.join(" ")));
        if DEBUG_BOOT {
            log_stdio("antes del fork");
        }

        let started = Instant::now();
        let shell = match start_shell(mode.args) {
            Ok(pid) => pid,
            Err(error) => {
                log(&format!("FATAL: fork() fallo: {:?}", error));
                std::process::exit(1);
            }
        };
        debug(&format!("shell lanzado, PID={}", shell));

        let code = match supervise(shell) {
            Outcome::Exited(code) => {
                log(&format!("EL SHELL TERMINO: codigo {} tras {:?}", code, started.elapsed()));
                code
            }
            Outcome::Signaled(signal) => {
                log(&format!("EL SHELL MURIO por {} tras {:?}", signal, started.elapsed()));
                if signal == "SIGSYS" {
                    log("SIGSYS = el seccomp de Android bloqueo un syscall que usa brush");
                }
                128
            }
            Outcome::Lost => {
                log("ERROR: el shell ya no existe (ECHILD)");
                127
            }
        };

        reclaim_tty();

        if code == 0 || !BOOT_MENU || attempts >= MAX_ATTEMPTS || started.elapsed() > QUICK_EXIT {
            log("Kyanite session is shutting down.");
            std::process::exit(code);
        }

        log("el shell fallo; vuelvo al menu para probar otro modo.");
    }
}

#[cfg(test)]
mod tests {
    use super::wants_passthrough;

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn command_and_script_pass_through() {
        assert!(wants_passthrough(&v(&["-c", "echo hola"]), false, true));
        assert!(wants_passthrough(&v(&["-c", "echo hola"]), false, false));
        assert!(wants_passthrough(&v(&["-ec", "exit 7"]), false, true));
        assert!(wants_passthrough(&v(&["script.sh", "a"]), false, true));
        assert!(wants_passthrough(&v(&["-e", "script.sh"]), false, true));
        assert!(wants_passthrough(&v(&["-s"]), false, true));
        assert!(wants_passthrough(&v(&["-"]), false, true));
        assert!(wants_passthrough(&v(&["--"]), false, true));
    }

    #[test]
    fn stdin_pipe_passes_through() {
        assert!(wants_passthrough(&v(&[]), false, false));
    }

    #[test]
    fn interactive_sessions_are_kept() {
        assert!(!wants_passthrough(&v(&[]), false, true));
        assert!(!wants_passthrough(&v(&["-l"]), false, true));
        assert!(!wants_passthrough(&v(&["-il"]), false, true));
        assert!(!wants_passthrough(&v(&["-l"]), false, false));
        assert!(!wants_passthrough(&v(&["--login"]), false, true));
    }

    #[test]
    fn pid1_never_passes_through() {
        assert!(!wants_passthrough(&v(&["-l"]), true, true));
        assert!(!wants_passthrough(&v(&[]), true, false));
        assert!(!wants_passthrough(&v(&["-c", "x"]), true, false));
    }
}
