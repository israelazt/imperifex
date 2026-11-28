//! Numbers with decimals, as each language writes them. No thousands separator.

use super::Language;

/// `value` rounded to `digits` decimals: a comma in Spanish, a point in English.
pub fn decimal(language: Language, value: f64, digits: usize) -> String {
    let written = format!("{value:.digits$}");
    match language {
        Language::Spanish => written.replace('.', ","),
        Language::English => written,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spanish_writes_a_comma_and_english_a_point() {
        assert_eq!(decimal(Language::Spanish, 5.25, 1), "5,2");
        assert_eq!(decimal(Language::English, 5.25, 1), "5.2");
        assert_eq!(decimal(Language::Spanish, -0.75, 2), "-0,75");
        assert_eq!(decimal(Language::Spanish, 1_234_567.0, 0), "1234567");
    }
}
