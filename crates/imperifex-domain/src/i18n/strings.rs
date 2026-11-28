//! Every text of the interface, in both languages side by side.

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
    ShellNotStarted {
        es: "No se pudo abrir la shell.",
        en: "The shell could not be opened.",
    },
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
    fn each_language_gets_its_own_words() {
        assert_eq!(text(Language::Spanish, Text::Close), "Cerrar");
        assert_eq!(text(Language::English, Text::Close), "Close");
    }
}
