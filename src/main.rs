use anyhow::Result;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::{env, path::PathBuf, process, sync::mpsc::channel};
use terminal_size::terminal_size;
use which::which;

fn main() {
    match try_main() {
        Ok(code) => process::exit(code),
        Err(err) => {
            println!("error: {err}");
            process::exit(1)
        }
    }
}

fn try_main() -> Result<i32> {
    let command;
    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "-h" | "--help" => return print_help(),
            "-V" | "--version" => return print_version(),
            _ => command = arg,
        }
    } else {
        return print_help();
    }

    let command = if !command.contains(std::path::MAIN_SEPARATOR) {
        which(command)?
    } else {
        PathBuf::from(command)
    };

    let pty_system = NativePtySystem::default();
    let (cols, rows) = term_size();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut command_builder = CommandBuilder::new(command);
    command_builder.args(env::args_os().skip(2));

    let mut child = pair.slave.spawn_command(command_builder)?;

    drop(pair.slave);

    let (tx, rx) = channel();
    let mut reader = pair.master.try_clone_reader()?;
    std::thread::spawn(move || {
        let mut s = String::new();
        reader.read_to_string(&mut s).unwrap();
        tx.send(s).unwrap();
    });

    let code = child.wait()?.exit_code() as i32;
    drop(pair.master);
    let output = rx.recv()?;
    print!("{output}");
    Ok(code)
}

fn term_size() -> (u16, u16) {
    let (cols, rows) = match terminal_size() {
        Some((width, height)) => (width.0, height.0),
        None => (80, 24),
    };
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(cols);
    let rows = std::env::var("LINES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(rows);
    (cols, rows)
}

fn print_help() -> Result<i32> {
    println!(
        r###"{}

Usage: {} <command> [args...]

Args:
  <command>                  command to run in a fake pty
  [args...]                  Args passed to command

Options:
  -h, --help                 Print help
  -V, --version              Print version
"###,
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_CRATE_NAME")
    );
    Ok(0)
}

fn print_version() -> Result<i32> {
    println!("{} {}", env!("CARGO_CRATE_NAME"), env!("CARGO_PKG_VERSION"));
    Ok(0)
}
