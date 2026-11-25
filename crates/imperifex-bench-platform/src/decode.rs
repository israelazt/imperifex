//! Turning what Windows returns into data. Plain code, tested everywhere.

/// The text of a wide buffer, up to its first zero or its end.
pub(crate) fn utf16(wide: &[u16]) -> String {
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

/// The 16 bit values of some little endian bytes; an odd last byte is left out.
pub(crate) fn words(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .collect()
}

/// The key of the first language of a version resource, from its translation
/// table (pairs of language and code page), such as `040904b0`.
pub(crate) fn language_key(table: &[u8]) -> Option<String> {
    match words(table)[..] {
        [language, code_page, ..] => Some(format!("{language:04x}{code_page:04x}")),
        _ => None,
    }
}

/// `YYYY-MM-DD`.
pub(crate) fn date_text(year: u16, month: u16, day: u16) -> String {
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_text_ends_at_its_zero_or_at_its_end() {
        assert_eq!(utf16(&[72, 105, 0, 99]), "Hi");
        assert_eq!(utf16(&[72, 105]), "Hi");
        assert_eq!(utf16(&[]), "");
    }

    #[test]
    fn bytes_are_read_as_little_endian_words() {
        assert_eq!(words(&[1, 0, 0, 1, 9]), [1, 256]);
    }

    #[test]
    fn the_first_language_of_the_table_names_the_strings() {
        let table = [0x09, 0x04, 0xB0, 0x04, 0x0A, 0x08, 0xB0, 0x04];
        assert_eq!(language_key(&table).as_deref(), Some("040904b0"));
        assert_eq!(language_key(&table[..3]), None);
    }

    #[test]
    fn a_date_has_four_two_and_two_digits() {
        assert_eq!(date_text(2026, 3, 7), "2026-03-07");
    }
}
