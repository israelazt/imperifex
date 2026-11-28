//! Every text of the interface, in both languages side by side. A `{}` is
//! where `fill` puts a value.

/// Declares `Text`, one key per entry, and `text`, which gives its words.
///
/// An entry needs both languages:
///
/// ```
/// imperifex_domain::strings! {
///     Hello { es: "Hola", en: "Hello" },
/// }
/// ```
///
/// Without one of them it does not compile:
///
/// ```compile_fail
/// imperifex_domain::strings! {
///     Hello { es: "Hola" },
/// }
/// ```
#[macro_export]
macro_rules! strings {
    ($($key:ident { es: $es:literal, en: $en:literal $(,)? }),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Text {
            $($key),+
        }

        impl Text {
            pub const ALL: &'static [Self] = &[$(Self::$key),+];
        }

        pub fn text(language: $crate::i18n::Language, key: Text) -> &'static str {
            match language {
                $crate::i18n::Language::Spanish => match key {
                    $(Text::$key => $es),+
                },
                $crate::i18n::Language::English => match key {
                    $(Text::$key => $en),+
                },
            }
        }
    };
}

crate::strings! {
    AppName { es: "Imperifex", en: "Imperifex" },
    Minimize { es: "Minimizar", en: "Minimize" },
    Maximize { es: "Maximizar", en: "Maximize" },
    Restore { es: "Restaurar", en: "Restore" },
    Close { es: "Cerrar", en: "Close" },
    SettingsDamaged {
        es: "El archivo de ajustes está dañado: lo que no se pudo leer usa su valor por defecto. Se guardó una copia.",
        en: "The settings file is damaged: what could not be read uses its default value. A copy was kept.",
    },
    SettingsLocked {
        es: "No se pudo leer o copiar el archivo de ajustes. No se escribirá sobre él.",
        en: "The settings file could not be read or copied. It will not be written over.",
    },
    FontNotFound {
        es: "No se encontró la fuente elegida; se usa Consolas.",
        en: "The chosen font was not found; Consolas is used.",
    },
    KeyEnter {
        es: "Enter",
        en: "Enter",
    },
    EditorRun {
        es: "ejecutar",
        en: "run",
    },
    KeyShiftEnter {
        es: "Mayús+Enter",
        en: "Shift+Enter",
    },
    EditorNewLine {
        es: "línea nueva",
        en: "new line",
    },
    KeyRight {
        es: "→",
        en: "→",
    },
    EditorAccept {
        es: "aceptar sugerencia",
        en: "accept suggestion",
    },
    KeyUp {
        es: "↑",
        en: "↑",
    },
    EditorHistory {
        es: "historial",
        en: "history",
    },
    EditorRefused {
        es: "Esta shell no admite órdenes de varias líneas.",
        en: "This shell does not take commands of several lines.",
    },
    PasteLines {
        es: "Pegar {} líneas las ejecutaría: pega otra vez para enviarlas.",
        en: "Pasting {} lines would run them: paste again to send them.",
    },
    CommandPrompt {
        es: "Símbolo del sistema",
        en: "Command Prompt",
    },
    ShellNotStarted {
        es: "No se pudo abrir la shell.",
        en: "The shell could not be opened.",
    },
    Help {
        es: "Imperifex: la terminal de desarrollo de la familia Fexarium

Uso:
  imperifex [window]          Abre la ventana
  imperifex status [--json]   Comprueba que la app funciona
  imperifex run --headless [--columns N] [--rows N] [--timeout S] [--history]
                -- <programa> [argumentos]
      Corre un programa en una consola sin ventana y escribe la pantalla que
      deja (80x25 y 30 s si no se dice otra cosa). --history escribe también
      lo que salió por arriba.
  imperifex run --headless --profile <perfil> [opciones] -- \"<orden>\"
      Lo mismo, con la orden escrita para una shell: pwsh, powershell, cmd o wsl.
  imperifex help              Esta ayuda
",
        en: "Imperifex: the development terminal of the Fexarium family

Usage:
  imperifex [window]          Opens the window
  imperifex status [--json]   Checks that the app works
  imperifex run --headless [--columns N] [--rows N] [--timeout S] [--history]
                -- <program> [arguments]
      Runs a program in a console with no window and prints the screen it
      leaves (80x25 and 30 s unless told otherwise). --history also prints
      what went off the top.
  imperifex run --headless --profile <profile> [options] -- \"<command>\"
      The same, with the command written for a shell: pwsh, powershell, cmd or wsl.
  imperifex help              This help
",
    },
    StatusWorks {
        es: "Imperifex {} funciona · {} ms",
        en: "Imperifex {} works · {} ms",
    },
    UnknownCommand {
        es: "orden desconocida: {}",
        en: "unknown command: {}",
    },
    UnknownOption {
        es: "opción desconocida: {}",
        en: "unknown option: {}",
    },
    MissingNumber {
        es: "falta el número de {}",
        en: "the number for {} is missing",
    },
    NumberOutOfRange {
        es: "{} pide un número de 1 a {}, no «{}»",
        en: "{} takes a number from 1 to {}, not “{}”",
    },
    UnknownProfile {
        es: "no hay un perfil «{}»; los que hay son: {}",
        en: "there is no “{}” profile; these are the ones: {}",
    },
    MissingProgram {
        es: "falta el programa: run --headless -- <programa> [argumentos]",
        en: "the program is missing: run --headless -- <program> [arguments]",
    },
    OneCommand {
        es: "con --profile va una sola orden entre comillas después de --",
        en: "with --profile, put one single command in quotes after --",
    },
    NeedsHeadless {
        es: "por ahora run solo corre sin ventana: usa run --headless",
        en: "for now run only works with no window: use run --headless",
    },
    ExitedWith {
        es: "el programa salió con el código {}",
        en: "the program left with code {}",
    },
    TimedOut {
        es: "no terminó en {} s: se cerró su consola",
        en: "it did not finish in {} s: its console was closed",
    },
    StillRunning {
        es: "sigue vivo después de cerrar su consola: proceso {} (no se termina)",
        en: "it is still running after its console was closed: process {} (it is not terminated)",
    },
    OutputCut {
        es: "la salida no terminó de llegar: puede faltar el final",
        en: "the output did not finish arriving: the end may be missing",
    },
    ShellMissing {
        es: "{} (¿está instalado?): {}",
        en: "{} (is it installed?): {}",
    },
}

/// The text of `key` with each `{}` replaced by the next of `values`. What a
/// value holds is never read as another place to fill.
pub fn fill(
    language: crate::i18n::Language,
    key: Text,
    values: &[&dyn std::fmt::Display],
) -> String {
    let mut parts = text(language, key).split("{}");
    let mut filled = parts.next().unwrap_or_default().to_string();
    let mut values = values.iter();
    for part in parts {
        if let Some(value) = values.next() {
            filled.push_str(&value.to_string());
        }
        filled.push_str(part);
    }
    filled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn no_text_is_empty_in_either_language() {
        for language in Language::ALL {
            for &key in Text::ALL {
                assert!(
                    !text(language, key).trim().is_empty(),
                    "{language:?} {key:?}"
                );
            }
        }
    }

    #[test]
    fn both_languages_of_a_text_have_the_same_places_to_fill() {
        let places = |language, key| text(language, key).matches("{}").count();
        for &key in Text::ALL {
            let spanish = places(Language::Spanish, key);
            assert_eq!(spanish, places(Language::English, key), "{key:?}");
        }
    }

    #[test]
    fn the_values_go_in_order_and_are_not_read_as_places() {
        let filled = fill(Language::English, Text::ShellMissing, &[&"{}", &7]);
        assert_eq!(filled, "{} (is it installed?): 7");
    }

    #[test]
    fn each_language_gets_its_own_words() {
        assert_eq!(text(Language::Spanish, Text::Close), "Cerrar");
        assert_eq!(text(Language::English, Text::Close), "Close");
    }
}
