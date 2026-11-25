//! The bench: it creates the real adapters, hands them to its services and
//! prints what they return. The decisions are not here. See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

#[cfg(windows)]
mod adapters;

#[cfg(windows)]
fn main() {
    use imperifex_bench_app::Command;
    use imperifex_platform::install_stderr_log;

    install_stderr_log();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match Command::parse(&args) {
        Ok(command) => command,
        Err(error) => {
            imperifex_domain::log_error!("{error}");
            std::process::exit(2);
        }
    };
    let output = command.run(&adapters::RealAdapters::default());
    println!("{}", output.text);
    std::process::exit(if output.ok { 0 } else { 1 });
}

/// The bench only has its system adapters for Windows so far.
#[cfg(not(windows))]
fn main() {
    imperifex_platform::install_stderr_log();
    imperifex_domain::log_error!("imperifex-bench does not have an adapter for this system yet.");
    std::process::exit(1);
}
