//! The language Windows speaks to the user: that of its interface, not the
//! format of dates and numbers.

/// The first name of a list as Windows gives it: names ended by a zero, one
/// after another, and another zero after the last.
#[cfg(any(windows, test))]
fn first_of_list(buffer: &[u16]) -> Option<String> {
    let end = buffer.iter().position(|&unit| unit == 0)?;
    let name = String::from_utf16(&buffer[..end]).ok()?;
    (!name.is_empty()).then_some(name)
}

#[cfg(windows)]
mod windows_list {
    use imperifex_domain::log_warn;
    use imperifex_domain::ports::SystemLanguage;
    use windows::core::PWSTR;
    use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};

    pub struct WindowsLanguage;

    impl SystemLanguage for WindowsLanguage {
        fn locale(&self) -> Option<String> {
            match preferred() {
                Ok(buffer) => super::first_of_list(&buffer),
                Err(error) => {
                    log_warn!("the system language is not known: {error}");
                    None
                }
            }
        }
    }

    /// The whole list, in a buffer of the size Windows asks for.
    fn preferred() -> windows::core::Result<Vec<u16>> {
        let (mut languages, mut length) = (0, 0);
        // SAFETY: with no buffer, the call only writes the sizes, into values
        // that live through it.
        unsafe {
            GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut languages, None, &mut length)?;
        }
        let mut buffer = vec![0u16; usize::try_from(length).unwrap_or(0)];
        let into = PWSTR(buffer.as_mut_ptr());
        // SAFETY: the buffer has the `length` units said.
        unsafe {
            GetUserPreferredUILanguages(
                MUI_LANGUAGE_NAME,
                &mut languages,
                Some(into),
                &mut length,
            )?;
        }
        Ok(buffer)
    }
}

#[cfg(windows)]
pub use windows_list::WindowsLanguage;

#[cfg(test)]
mod tests {
    use super::*;

    fn units(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn the_first_of_the_list_is_the_language() {
        let list = units("es-MX\0en-US\0\0");
        assert_eq!(first_of_list(&list), Some("es-MX".into()));
    }

    #[test]
    fn an_empty_cut_or_broken_list_gives_no_language() {
        assert_eq!(first_of_list(&[]), None);
        assert_eq!(first_of_list(&units("\0\0")), None);
        assert_eq!(first_of_list(&units("es-MX")), None);
        assert_eq!(first_of_list(&[0xD800, 0, 0]), None);
    }
}
