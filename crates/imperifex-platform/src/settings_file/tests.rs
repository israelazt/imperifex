//! In a folder of their own under the temporary one: never in `%APPDATA%`.

use super::*;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let id = std::process::id();
        let folder = std::env::temp_dir().join(format!("imperifex-settings-{id}-{name}"));
        let _ = fs::remove_dir_all(&folder);
        Self(folder)
    }

    fn store(&self) -> FileSettingsStore {
        FileSettingsStore::new(self.0.join("Imperifex"))
    }

    fn bytes(&self, name: &str) -> Vec<u8> {
        fs::read(self.0.join("Imperifex").join(name)).unwrap()
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.0.join("Imperifex"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn with_no_file_there_is_nothing_and_no_folder_is_made() {
    let scratch = Scratch::new("none");
    assert_eq!(scratch.store().load(), Ok(None));
    assert!(!scratch.0.exists());
}

#[test]
fn saving_creates_the_folder_replaces_the_file_and_leaves_no_temporary() {
    let scratch = Scratch::new("save");
    let store = scratch.store();
    store.save("uno").unwrap();
    store.save("dos").unwrap();
    assert_eq!(store.load(), Ok(Some("dos".into())));
    assert_eq!(scratch.names(), ["settings.json"]);
}

#[test]
fn each_different_damaged_file_gets_its_own_copy_and_a_repeated_one_does_not() {
    let scratch = Scratch::new("copies");
    let store = scratch.store();
    store.save("{").unwrap();
    store.keep_damaged("{").unwrap();
    store.keep_damaged("{").unwrap();
    store.save("[").unwrap();
    let kept = store.keep_damaged("[").unwrap();
    assert!(kept.ends_with("settings.damaged-2.json"), "{kept}");
    assert_eq!(scratch.bytes("settings.damaged.json"), b"{");
    assert_eq!(scratch.bytes("settings.damaged-2.json"), b"[");
    assert_eq!(scratch.names().len(), 3);
}

#[test]
fn a_copy_keeps_the_bytes_that_are_not_text() {
    let scratch = Scratch::new("bytes");
    let store = scratch.store();
    store.save("").unwrap();
    let bytes = [b'{', 0xFF, 0xFE, b'}'];
    fs::write(scratch.0.join("Imperifex").join(FILE), bytes).unwrap();
    let text = store.load().unwrap().unwrap();
    assert!(text.contains('\u{FFFD}'));
    store.keep_damaged(&text).unwrap();
    assert_eq!(scratch.bytes("settings.damaged.json"), bytes);
}
