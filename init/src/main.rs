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

/// Arranque real de una imagen x86_64 (init es PID 1 y el kernel dejo HOME=/): home de root.
const X86_HOME: &str = "/root";
/// PATH que se da al shell en ese caso, solo si el kernel no dejo ninguno definido.
const X86_DEFAULT_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

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

/// true = stdin es un tty pero este proceso no tiene terminal de control (arranque VM).
/// tcgetpgrp() da ENOTTY cuando el tty no es el terminal de control del proceso que llama.
/// En proot el pty de la app ya es el terminal de control: da false y no se toca nada.
fn lacks_controlling_tty() -> bool {
    unsafe { libc::isatty(0) == 1 && libc::tcgetpgrp(0) < 0 }
}

/// true = el tty no conoce su tamano (consola serie en la VM: 0 filas x 0 columnas).
fn winsize_unknown() -> bool {
    unsafe {
        if libc::isatty(0) != 1 {
            return false;
        }
        let mut ws: libc::winsize = std::mem::zeroed();
        libc::ioctl(0, libc::TIOCGWINSZ as _, &mut ws) != 0 || ws.ws_row == 0 || ws.ws_col == 0
    }
}

/// Se ejecuta en el hijo ya tras setsid(). Convierte el tty de stdin en terminal de control.
unsafe fn acquire_ctty() -> bool {
    // 1) stdin ya es la consola (/dev/ttyAMA0)
    if libc::isatty(0) == 1 && libc::ioctl(0, libc::TIOCSCTTY as _, 1) == 0 {
        return true;
    }
    // 2) plan B: abrir /dev/console y ponerlo en 0, 1 y 2
    let fd = libc::open(b"/dev/console\0".as_ptr() as *const libc::c_char, libc::O_RDWR);
    if fd < 0 {
        return false;
    }
    if libc::ioctl(fd, libc::TIOCSCTTY as _, 1) != 0 {
        libc::close(fd);
        return false;
    }
    for target in 0..3 {
        libc::dup2(fd, target);
    }
    if fd > 2 {
        libc::close(fd);
    }
    true
}

/// Pregunta al emulador su tamano (ESC[999;999H + ESC[6n) y lo aplica al tty con TIOCSWINSZ.
/// Una consola serie (ttyAMA0) no conoce el tamano de la ventana; sin esto el shell asume 0x0 / 24x80.
fn autodetect_winsize() {
    unsafe {
        if libc::isatty(0) != 1 {
            return;
        }
        let mut old: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(0, &mut old) != 0 {
            return;
        }
        let mut raw = old;
        libc::cfmakeraw(&mut raw);
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 5; // 0,5 s por read
        if libc::tcsetattr(0, libc::TCSANOW, &raw) != 0 {
            return;
        }

        let query = b"\x1b7\x1b[999;999H\x1b[6n\x1b8";
        libc::write(1, query.as_ptr() as *const libc::c_void, query.len());

        let mut buf = [0u8; 64];
        let mut len = 0usize;
        for _ in 0..4 {
            let n = libc::read(0, buf[len..].as_mut_ptr() as *mut libc::c_void, buf.len() - len);
            if n > 0 {
                len += n as usize;
            }
            if buf[..len].contains(&b'R') || len == buf.len() {
                break;
            }
        }
        libc::tcsetattr(0, libc::TCSANOW, &old);

        if let Some((rows, cols)) = parse_cpr(&buf[..len]) {
            let ws = libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 };
            libc::ioctl(0, libc::TIOCSWINSZ as _, &ws);
            debug(&format!("tamano de ventana detectado: {} filas x {} columnas", rows, cols));
        } else {
            debug("el terminal no respondio a ESC[6n: tamano sin cambiar");
        }
    }
}

/// Respuesta del terminal: ESC [ fila ; columna R
fn parse_cpr(buf: &[u8]) -> Option<(u16, u16)> {
    let start = buf.windows(2).position(|w| w == b"\x1b[")? + 2;
    let end = start + buf[start..].iter().position(|&b| b == b'R')?;
    let text = std::str::from_utf8(&buf[start..end]).ok()?;
    let (rows, cols) = text.split_once(';')?;
    let (rows, cols): (u16, u16) = (rows.parse().ok()?, cols.parse().ok()?);
    if rows == 0 || cols == 0 {
        None
    } else {
        Some((rows, cols))
    }
}

/// Se ejecuta en el hijo, justo antes del exec.
fn child_job_control(vm: bool) {
    unsafe {
        let pid = libc::getpid();

        if vm {
            // setsid() deja al hijo como lider de sesion y de grupo (pgrp = pid).
            if libc::setsid() < 0 {
                log(&format!("setsid() FALLO: {}", io::Error::last_os_error()));
            } else if !acquire_ctty() {
                log("no se pudo adquirir el terminal de control");
            }
        } else if libc::setpgid(0, 0) != 0 {
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

// ------------------------------------------------------------
// ARRANQUE REAL x86_64 (imagen booteable)
// ------------------------------------------------------------
// Cuando el kernel arranca la imagen, init es el PID 1 y recibe un entorno minimo (HOME=/ y
// TERM=linux). Brush, como shell de login, buscaria entonces ~/.bashrc y los perfiles en "/" en
// vez de /root. Solo en ese caso (x86_64 + PID 1 + HOME=/) se prepara el entorno del shell.
// En proot (init no es PID 1), en la VM aarch64 o con cualquier otro HOME no se cambia nada.

/// Logica pura (testeable): true = arranque real x86_64 con el HOME por defecto del kernel.
fn is_fresh_x86_boot(
    arch_is_x86_64: bool,
    is_pid1: bool,
    home: Option<&str>,
    root_home_exists: bool,
) -> bool {
    arch_is_x86_64 && is_pid1 && home == Some("/") && root_home_exists
}

fn needs_x86_boot_env(is_pid1: bool) -> bool {
    let home = std::env::var("HOME").ok();
    is_fresh_x86_boot(
        cfg!(target_arch = "x86_64"),
        is_pid1,
        home.as_deref(),
        std::path::Path::new(X86_HOME).is_dir(),
    )
}

/// Variables que se anaden al entorno del shell en ese caso.
fn x86_boot_env() -> Vec<(&'static str, String)> {
    let mut env = vec![
        ("HOME", X86_HOME.to_string()),
        ("USER", "root".to_string()),
        ("LOGNAME", "root".to_string()),
    ];
    if std::env::var_os("PATH").is_none() {
        env.push(("PATH", X86_DEFAULT_PATH.to_string()));
    }
    env
}

fn start_shell(args: &[&str], x86_boot: bool) -> Result<Pid, Errno> {
    let _ = io::stdout().flush();
    let _ = io::stderr().flush();

    // Se calcula antes del fork para no leer el entorno dentro del hijo.
    let extra_env = if x86_boot { x86_boot_env() } else { Vec::new() };

    match unsafe { fork()? } {
        ForkResult::Child => {
            child_job_control(lacks_controlling_tty());

            let mut command = Command::new(SHELL_PATH);
            command.args(args).env("SHELL", SHELL_PATH);
            for (name, value) in &extra_env {
                command.env(name, value);
            }
            let error = command.exec();

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

    // Arranque real de una imagen x86_64: el kernel dejo HOME=/ (ver ARRANQUE REAL x86_64).
    // En cualquier otro caso (proot, VM aarch64...) x86_boot es false y todo va como antes.
    let x86_boot = needs_x86_boot_env(std::process::id() == 1);
    if x86_boot {
        let _ = std::env::set_current_dir(X86_HOME);
        debug("arranque x86_64 con HOME=/: se usa HOME=/root");
    }
    ignore_sigttou();
    if winsize_unknown() {
        autodetect_winsize();
    }

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
        let shell = match start_shell(mode.args, x86_boot) {
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

        if !lacks_controlling_tty() {
            reclaim_tty();
        }

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
    fn x86_boot_fix_only_applies_to_a_fresh_x86_pid1() {
        assert!(super::is_fresh_x86_boot(true, true, Some("/"), true));
        assert!(!super::is_fresh_x86_boot(false, true, Some("/"), true)); // aarch64 (VM del movil)
        assert!(!super::is_fresh_x86_boot(true, false, Some("/"), true)); // proot / uso como sh
        assert!(!super::is_fresh_x86_boot(true, true, Some("/root"), true)); // HOME ya definido
        assert!(!super::is_fresh_x86_boot(true, true, None, true));
        assert!(!super::is_fresh_x86_boot(true, true, Some("/"), false)); // no existe /root
    }

    #[test]
    fn cpr_is_parsed() {
        assert_eq!(super::parse_cpr(b"\x1b[62;90R"), Some((62, 90)));
        assert_eq!(super::parse_cpr(b"basura\x1b[11;1Rmas"), Some((11, 1)));
        assert_eq!(super::parse_cpr(b"\x1b[0;0R"), None);
        assert_eq!(super::parse_cpr(b""), None);
    }

    #[test]
    fn pid1_never_passes_through() {
        assert!(!wants_passthrough(&v(&["-l"]), true, true));
        assert!(!wants_passthrough(&v(&[]), true, false));
        assert!(!wants_passthrough(&v(&["-c", "x"]), true, false));
    }
}
