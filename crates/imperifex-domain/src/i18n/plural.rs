//! A number with its noun: "1 agente", "2 agentes".

use super::Language;

/// What can be counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Noun {
    Agent,
}

impl Noun {
    /// The singular and the plural.
    fn forms(self, language: Language) -> [&'static str; 2] {
        match (self, language) {
            (Self::Agent, Language::Spanish) => ["agente", "agentes"],
            (Self::Agent, Language::English) => ["agent", "agents"],
        }
    }
}

/// Only one is singular, in both languages: zero is plural.
pub fn count(language: Language, n: u64, noun: Noun) -> String {
    let [one, many] = noun.forms(language);
    let word = if n == 1 { one } else { many };
    format!("{n} {word}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_is_singular() {
        let spanish = |n| count(Language::Spanish, n, Noun::Agent);
        assert_eq!(spanish(0), "0 agentes");
        assert_eq!(spanish(1), "1 agente");
        assert_eq!(spanish(2), "2 agentes");
        let english = |n| count(Language::English, n, Noun::Agent);
        assert_eq!(english(0), "0 agents");
        assert_eq!(english(1), "1 agent");
        assert_eq!(english(21), "21 agents");
    }
}
