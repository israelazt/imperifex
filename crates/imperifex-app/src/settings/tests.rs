use std::cell::RefCell;
use std::rc::Rc;

use imperifex_domain::i18n::Language;
use imperifex_domain::theme::ThemeChoice;

use super::*;
use crate::testing::{FakeStore, Written};

fn load(store: FakeStore) -> (SettingsService, Option<Notice>, Rc<RefCell<Written>>) {
    let written = Rc::clone(&store.written);
    let (service, notice) = SettingsService::load(Box::new(store));
    (service, notice, written)
}

fn with_file(text: &str) -> FakeStore {
    FakeStore {
        file: Some(text.into()),
        ..FakeStore::default()
    }
}

fn dark() -> Settings {
    Settings {
        theme: ThemeChoice::Dark,
        ..Settings::default()
    }
}

#[test]
fn with_no_file_the_defaults_are_used_and_nothing_is_written() {
    let (mut service, notice, written) = load(FakeStore::default());
    assert_eq!(service.settings(), &Settings::default());
    assert_eq!(notice, None);
    service.save(Settings::default()).unwrap();
    assert!(written.borrow().saved.is_empty());
    assert!(written.borrow().copies.is_empty());
}

#[test]
fn only_a_change_is_written_and_only_once() {
    let (mut service, _, written) = load(FakeStore::default());
    service.save(dark()).unwrap();
    service.save(dark()).unwrap();
    assert_eq!(written.borrow().saved, [dark().write()]);
    assert_eq!(service.settings(), &dark());
}

#[test]
fn a_damaged_file_is_copied_as_it_was_and_said() {
    let text = r#"{ "theme": "dark", "shell": 3 }"#;
    let (service, notice, written) = load(with_file(text));
    assert_eq!(service.settings(), &dark());
    let kept = "la-copia.json".to_string();
    assert_eq!(notice, Some(Notice::Damaged { kept }));
    assert_eq!(written.borrow().copies, [text]);
    assert!(written.borrow().saved.is_empty());
}

#[test]
fn a_file_that_cannot_be_read_is_never_written_over() {
    let store = FakeStore {
        load_fails: true,
        ..FakeStore::default()
    };
    let (mut service, notice, written) = load(store);
    assert!(matches!(notice, Some(Notice::Locked { .. })));
    assert!(service.save(dark()).is_err());
    assert!(written.borrow().saved.is_empty());
}

#[test]
fn a_damaged_file_that_cannot_be_copied_is_never_written_over() {
    let store = FakeStore {
        copy_fails: true,
        ..with_file("{")
    };
    let (mut service, notice, written) = load(store);
    assert!(matches!(notice, Some(Notice::Locked { .. })));
    assert!(service.save(dark()).is_err());
    assert!(written.borrow().saved.is_empty());
}

#[test]
fn a_failed_write_is_an_error_and_can_be_tried_again() {
    let store = FakeStore {
        save_fails: true,
        ..FakeStore::default()
    };
    let (mut service, _, _) = load(store);
    assert!(service.save(dark()).is_err());
    assert_eq!(service.settings(), &Settings::default());
    assert!(service.save(dark()).is_err());
}

#[test]
fn a_notice_is_said_in_the_users_language_with_its_detail() {
    let kept = "la-copia.json".to_string();
    let said = Notice::Damaged { kept }.text(Language::English);
    assert!(said.starts_with("The settings file is damaged"), "{said}");
    assert!(said.ends_with("(la-copia.json)"), "{said}");
}
