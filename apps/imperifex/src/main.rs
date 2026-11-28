//! Imperifex: the program. It creates the real adapters, hands them to the
//! application and prints what it returns. The decisions are not here.
//! See `ARCHITECTURE.md`.

#![forbid(unsafe_code)]

#[cfg(windows)]
mod window;

/// The settings, and the language of what is printed and shown: the one chosen
/// in them, or the system's. What is wrong with their file is said in it.
#[cfg(windows)]
fn settings() -> (
    imperifex_domain::settings::Settings,
    imperifex_domain::i18n::Language,
) {
    use imperifex_app::settings::SettingsService;
    use imperifex_domain::i18n::resolve_language;
    use imperifex_domain::ports::SystemLanguage;
    use imperifex_platform::{FileSettingsStore, WindowsLanguage};

    let loaded = match FileSettingsStore::in_app_data() {
        Ok(store) => Some(SettingsService::load(Box::new(store))),
        Err(error) => {
            imperifex_domain::log_warn!("no settings: {error}");
            None
        }
    };
    let settings = loaded
        .as_ref()
        .map(|(service, _)| service.settings().clone())
        .unwrap_or_default();
    let language = resolve_language(settings.language, WindowsLanguage.locale().as_deref());
    if let Some(notice) = loaded.and_then(|(_, notice)| notice) {
        imperifex_domain::log_warn!("{}", notice.text(language));
    }
    (settings, language)
}

#[cfg(windows)]
fn main() {
    use imperifex_app::{Command, Shells, StatusService};
    use imperifex_platform::{install_stderr_log, ConPtyLauncher, SystemClock, VteParser};

    install_stderr_log();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (settings, language) = settings();
    let command = match Command::parse(&args, language) {
        Ok(command) => command,
        Err(error) => {
            imperifex_domain::log_error!("{error}");
            std::process::exit(2);
        }
    };
    if command == Command::Window {
        if let Err(error) = window::run(settings, language) {
            imperifex_domain::log_error!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let status = StatusService::new(SystemClock::new());
    let shells = Shells {
        launcher: &ConPtyLauncher,
        parser: &mut VteParser::new(),
        clock: &SystemClock::new(),
    };
    let output = command.run(&status, shells, language);
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
