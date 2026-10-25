//! Runs a command on a display of its own, which `cargo testbox vnc` shows: TigerVNC's Xvnc,
//! served on a Unix socket only, with the viewer's input and the clipboard off. (After the
//! testbox of barafael/beep, `testbox/tools/src/bin/with-display.rs`.)
//!
//! `with-display <socket> <width>x<height> -- <command> [argument...]`
//!
//! As the container's entrypoint it is PID 1, which ignores a signal it has no handler for: it
//! hands SIGTERM, SIGINT and SIGHUP on to the command, so that `podman stop` and `restart` let
//! the game end cleanly (its record flushed) instead of killing it 10 s later.

use std::io::{BufRead, BufReader};
use std::process::{Command, ExitCode, Stdio};

const USAGE: &str = "with-display <socket> <width>x<height> -- <command> [argument...]";
/// The container's one X display.
const DISPLAY: &str = ":1";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("with-display: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [socket, geometry, separator, program, program_arguments @ ..] = arguments.as_slice()
    else {
        return Err(format!("usage: {USAGE}"));
    };
    if separator != "--" {
        return Err(format!("usage: {USAGE}"));
    }
    // One left by this container before it restarted: Xvnc doesn't bind over it.
    match std::fs::remove_file(socket) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{socket}: {error}")),
    }
    let mut xvnc = Command::new("Xvnc")
        .args([DISPLAY, "-geometry", geometry, "-depth", "24"])
        // The socket only, the owner's only: no VNC port, no X over TCP.
        .args(["-rfbport=-1", "-rfbunixpath", socket, "-rfbunixmode", "0600"])
        // X only on the file socket in this container's /tmp: the pod's containers share a
        // network namespace, so an abstract socket would let them all reach this display, and
        // collide with each other's `:1`.
        .args(["-nolisten", "tcp", "-nolisten", "local"])
        // Whoever can open the socket may watch: its mode and its directory's are the access
        // control.
        .args(["-SecurityTypes", "None", "-AlwaysShared"])
        .args(["-AcceptKeyEvents=0", "-AcceptPointerEvents=0", "-AcceptSetDesktopSize=0"])
        .args(["-AcceptCutText=0", "-SendCutText=0", "-SendPrimary=0", "-SetPrimary=0"])
        // The display's number, once it accepts connections (Xserver(1) `-displayfd`).
        .args(["-displayfd", "1"])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Xvnc: {error}"))?;
    let stdout = xvnc.stdout.take().ok_or("Xvnc's stdout isn't piped")?;
    // Open while Xvnc runs: closed, Xvnc's next write to it would end Xvnc (SIGPIPE).
    let mut announced = BufReader::new(stdout);
    let mut number = String::new();
    announced
        .read_line(&mut number)
        .map_err(|error| format!("Xvnc's -displayfd: {error}"))?;
    if number.trim().is_empty() {
        let status = xvnc.wait().map_err(|error| format!("Xvnc: {error}"))?;
        return Err(format!("Xvnc exited before it served a display: {status}"));
    }
    let mut command = Command::new(program)
        .args(program_arguments)
        .env("DISPLAY", DISPLAY)
        .spawn()
        .map_err(|error| format!("{program}: {error}"))?;
    hand_on_signals(command.id())?;
    let status = command.wait().map_err(|error| format!("{program}: {error}"))?;
    xvnc.kill().map_err(|error| format!("Xvnc: {error}"))?;
    xvnc.wait().map_err(|error| format!("Xvnc: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited: {status}"))
    }
}

/// Hands the stop signals this process gets on to `pid`, from a thread of its own, for as long as
/// this process runs.
fn hand_on_signals(pid: u32) -> Result<(), String> {
    use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
    let mut signals = signal_hook::iterator::Signals::new([SIGTERM, SIGINT, SIGHUP])
        .map_err(|error| format!("the stop signals' handler: {error}"))?;
    let pid = libc::pid_t::try_from(pid).map_err(|_| format!("pid {pid}"))?;
    std::thread::Builder::new()
        .name("signals".into())
        .spawn(move || {
            for signal in signals.forever() {
                // SAFETY: kill(2) takes plain integers and touches no memory of ours.
                if unsafe { libc::kill(pid, signal) } != 0 {
                    eprintln!(
                        "with-display: signal {signal} to {pid}: {}",
                        std::io::Error::last_os_error()
                    );
                }
            }
        })
        .map_err(|error| format!("the stop signals' thread: {error}"))?;
    Ok(())
}
