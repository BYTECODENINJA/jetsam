//! `jetsam-helper` entrypoint: parse argv, verify root, validate, run, emit one
//! JSON line, exit with the pacman status.

use std::io::{self, Write};

use jetsam_helper::{run, validate_remove_names, Action, HelperError, HelperResult, MAX_PACKAGES};

const USAGE: &str = "\
jetsam-helper - privileged Jetsam pacman operation

USAGE:
    jetsam-helper remove <package>...   Remove packages (--noconfirm -R)
    jetsam-helper upgrade               Full system upgrade (--noconfirm -Syu)

MUST be invoked through pkexec. Refuses to operate on protected packages
(pacman, glibc, systemd, kernels, drivers, base, ...) and rejects any name
that is not a strict Arch package identifier.
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if matches!(
        args.first().map(String::as_str),
        Some("-h" | "--help" | "help")
    ) {
        print!("{USAGE}");
        std::process::exit(0);
    }

    if let Err(err) = dispatch(&args) {
        let _ = writeln!(io::stderr(), "jetsam-helper: {}", err.message());
        std::process::exit(err.exit_code());
    }
}

fn dispatch(args: &[String]) -> Result<(), HelperError> {
    if !jetsam_helper::current_is_root() {
        return Err(HelperError::NotRoot);
    }
    let Some(action) = args.first() else {
        return Err(HelperError::NoPackages);
    };
    match action.as_str() {
        "remove" => {
            if args.len() > MAX_PACKAGES {
                return Err(HelperError::TooManyPackages(args.len()));
            }
            let names = validate_remove_names(&args[1..])?;
            let result = run(&Action::Remove, &names);
            emit(&result);
            exit_with(&result)
        }
        "upgrade" => {
            let result = run(&Action::Upgrade, &[]);
            emit(&result);
            exit_with(&result)
        }
        other => Err(HelperError::InvalidPackage(other.to_string())),
    }
}

fn emit(result: &HelperResult) {
    let mut line = serde_json::to_string(result).expect("result is serializable");
    line.push('\n');
    // Stdout is the only channel a pkexec caller sees on success.
    let _ = io::stdout().flush();
    let mut stdout = io::stdout();
    let _ = stdout.write_all(line.as_bytes());
    let _ = stdout.flush();
    // Mirror meaningful failure detail to stderr for humans.
    if !result.ok {
        let _ = writeln!(io::stderr(), "jetsam-helper: {}", result.stderr.trim());
    }
}

fn exit_with(result: &HelperResult) -> ! {
    std::process::exit(result.exit_code.unwrap_or(127))
}
