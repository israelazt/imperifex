use super::*;

fn chosen() -> Settings {
    Settings {
        theme: ThemeChoice::Light,
        language: LanguageChoice::English,
        font: FontSettings {
            family: "Consolas".into(),
            size: 11.5,
        },
        shell: Some("cmd".into()),
        adjust_contrast: false,
        ..Settings::default()
    }
}

#[test]
fn the_defaults_follow_the_system_and_use_cascadia_mono_at_12_points() {
    let settings = Settings::default();
    assert_eq!(settings.theme, ThemeChoice::System);
    assert_eq!(settings.language, LanguageChoice::System);
    assert_eq!(settings.font.family, "Cascadia Mono");
    assert_eq!(settings.font.size, 12.0);
    assert_eq!(settings.shell, None);
    assert!(settings.adjust_contrast);
    assert_eq!(settings.version, 1);
}

#[test]
fn what_is_written_is_read_back_the_same() {
    for settings in [Settings::default(), chosen()] {
        let read = Settings::read(&settings.write());
        assert_eq!(
            read,
            Read {
                settings,
                damaged: false
            }
        );
    }
}

#[test]
fn the_names_in_the_file_are_a_contract() {
    let value: Value = serde_json::from_str(&chosen().write()).unwrap();
    let expected = serde_json::json!({
        "version": 1,
        "theme": "light",
        "language": "english",
        "font": { "family": "Consolas", "size": 11.5 },
        "shell": "cmd",
        "adjust_contrast": false,
    });
    assert_eq!(value, expected);
}

#[test]
fn an_old_file_with_less_and_a_newer_one_with_more_are_not_damaged() {
    let old = Settings::read(r#"{ "theme": "dark" }"#);
    assert!(!old.damaged);
    assert_eq!(old.settings.theme, ThemeChoice::Dark);
    assert_eq!(old.settings.font, FontSettings::default());
    let newer = Settings::read(r#"{ "version": 7, "tabs": [1], "font": { "weight": 3 } }"#);
    assert_eq!(
        newer,
        Read {
            settings: Settings::default(),
            damaged: false
        }
    );
}

#[test]
fn a_byte_order_mark_is_not_damage() {
    let read = Settings::read("\u{FEFF}{ \"theme\": \"light\" }");
    assert!(!read.damaged);
    assert_eq!(read.settings.theme, ThemeChoice::Light);
}

#[test]
fn what_is_not_a_json_object_gives_the_defaults_and_is_damaged() {
    for text in ["", "{", "not json", "[]", "null", "12", "{} {}"] {
        let read = Settings::read(text);
        assert!(read.damaged, "{text:?}");
        assert_eq!(read.settings, Settings::default(), "{text:?}");
    }
}

#[test]
fn a_wrong_field_takes_its_default_and_the_rest_are_kept() {
    let wrong = [
        ("theme", serde_json::json!("blue")),
        ("theme", serde_json::json!(1)),
        ("language", serde_json::json!("french")),
        ("shell", serde_json::json!("  ")),
        ("shell", serde_json::json!([])),
        ("adjust_contrast", serde_json::json!("yes")),
        ("version", serde_json::json!("1")),
        ("font", serde_json::json!("Consolas")),
    ];
    for (field, value) in wrong {
        let mut file: Value = serde_json::from_str(&chosen().write()).unwrap();
        file[field] = value.clone();
        let read = Settings::read(&file.to_string());
        assert!(read.damaged, "{field} = {value}");
        let defaults = serde_json::to_value(Settings::default()).unwrap();
        let mut expected = serde_json::to_value(chosen()).unwrap();
        expected[field] = defaults[field].clone();
        assert_eq!(
            serde_json::to_value(read.settings).unwrap(),
            expected,
            "{field} = {value}"
        );
    }
}

#[test]
fn a_font_size_from_6_to_72_is_accepted_and_no_other() {
    let read = |size: &str| Settings::read(&format!(r#"{{ "font": {{ "size": {size} }} }}"#));
    for size in ["6", "72", "10.5"] {
        assert!(!read(size).damaged, "{size}");
    }
    for size in ["5.9", "72.1", "0", "-12", "1e40", "\"12\""] {
        let found = read(size);
        assert!(found.damaged, "{size}");
        assert_eq!(found.settings.font.size, 12.0, "{size}");
    }
}

#[test]
fn a_wrong_font_family_keeps_the_chosen_size() {
    let read = Settings::read(r#"{ "font": { "family": "", "size": 20 } }"#);
    assert!(read.damaged);
    assert_eq!(read.settings.font.family, "Cascadia Mono");
    assert_eq!(read.settings.font.size, 20.0);
}

#[test]
fn bytes_that_were_not_text_are_damage_even_inside_valid_json() {
    let read = Settings::read("{ \"shell\": \"cm\u{FFFD}\" }");
    assert!(read.damaged);
}
