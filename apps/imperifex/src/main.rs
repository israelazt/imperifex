//! Imperifex: the program. It creates the real adapters, hands them to the
//! application and prints what it returns. The decisions are not here.
//! See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

#[cfg(windows)]
fn main() {
    use imperifex_app::{Command, Shells, StatusService};
    use imperifex_platform::{install_stderr_log, ConPtyLauncher, SystemClock, VteParser};

    install_stderr_log();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match Command::parse(&args) {
        Ok(command) => command,
        Err(error) => {
            imperifex_domain::log_error!("{error}");
            std::process::exit(2);
        }
    };
    let status = StatusService::new(SystemClock::new());
    let shells = Shells {
        launcher: &ConPtyLauncher,
        parser: &mut VteParser::new(),
        clock: &SystemClock::new(),
    };
    let output = command.run(&status, shells);
    if !output.text.is_empty() {
        println!("{}", output.text);
    }
    if let Some(notice) = &output.notice {
        imperifex_domain::log_warn!("{notice}");
    }
    std::process::exit(if output.ok { 0 } else { 1 });
}

/// Imperifex only has its system adapters for Windows so far.
#[cfg(not(windows))]
fn main() {
    imperifex_platform::install_stderr_log();
    imperifex_domain::log_error!("Imperifex does not have an adapter for this system yet.");
    std::process::exit(1);
}
