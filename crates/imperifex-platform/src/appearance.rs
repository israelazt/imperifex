//! Light or dark, as Windows keeps it in the registry, and the notice when it
//! changes. It does not need a window.

use std::ffi::c_void;
use std::sync::mpsc;
use std::thread;

use imperifex_domain::log_warn;
use imperifex_domain::ports::{SystemAppearance, Wake};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_FILE_NOT_FOUND, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Registry::{
    RegCloseKey, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER,
    KEY_NOTIFY, REG_NOTIFY_CHANGE_LAST_SET, RRF_RT_REG_DWORD,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};

const KEY: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
/// 0 when applications are to be dark.
const VALUE: PCWSTR = w!("AppsUseLightTheme");

pub struct WindowsAppearance;

impl SystemAppearance for WindowsAppearance {
    /// Dark when Windows does not say.
    fn dark(&self) -> Result<bool, String> {
        let mut light = 0u32;
        let mut bytes = u32::try_from(size_of::<u32>()).unwrap_or(4);
        let into: *mut u32 = &mut light;
        // SAFETY: only a DWORD is accepted, `into` has room for one, `bytes`
        // says how much, and both live through the call.
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                KEY,
                VALUE,
                RRF_RT_REG_DWORD,
                None,
                Some(into.cast::<c_void>()),
                Some(&mut bytes),
            )
        };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(true);
        }
        result
            .ok()
            .map_err(|e| format!("RegGetValueW AppsUseLightTheme: {e}"))?;
        Ok(light == 0)
    }

    fn watch(&self, changed: Wake) -> Result<(), String> {
        let (started, starting) = mpsc::channel();
        thread::Builder::new()
            .name("appearance".into())
            .spawn(move || watch_the_key(&started, &changed))
            .map_err(|e| format!("no se pudo crear el hilo del tema: {e}"))?;
        starting
            .recv()
            .map_err(|_| "el hilo del tema terminó antes de empezar".to_string())?
    }
}

/// Runs on its own thread until the program ends: Windows ties the request for
/// a notice to the thread that made it.
fn watch_the_key(started: &mpsc::Sender<Result<(), String>>, changed: &Wake) {
    let watch = match Watch::new().and_then(|watch| watch.ask().map(|()| watch)) {
        Ok(watch) => watch,
        Err(error) => {
            let _ = started.send(Err(error));
            return;
        }
    };
    let _ = started.send(Ok(()));
    loop {
        // Asked again before telling, so that no change falls in between.
        if let Err(error) = watch.wait().and_then(|()| watch.ask()) {
            log_warn!("the system theme is no longer watched: {error}");
            return;
        }
        changed();
    }
}

/// The open key and the event Windows signals when it changes.
struct Watch {
    key: Key,
    event: Event,
}

impl Watch {
    fn new() -> Result<Self, String> {
        Ok(Self {
            key: Key::open()?,
            event: Event::new()?,
        })
    }

    /// Asks for one notice, of the next change of a value of the key.
    fn ask(&self) -> Result<(), String> {
        // SAFETY: the key and the event are open, and live as long as `self`.
        let result = unsafe {
            RegNotifyChangeKeyValue(
                self.key.0,
                false,
                REG_NOTIFY_CHANGE_LAST_SET,
                Some(self.event.0),
                true,
            )
        };
        result
            .ok()
            .map_err(|e| format!("RegNotifyChangeKeyValue: {e}"))
    }

    fn wait(&self) -> Result<(), String> {
        // SAFETY: the event is open.
        let waited = unsafe { WaitForSingleObject(self.event.0, INFINITE) };
        if waited == WAIT_OBJECT_0 {
            Ok(())
        } else {
            Err(format!("WaitForSingleObject: 0x{:08x}", waited.0))
        }
    }
}

struct Key(HKEY);

impl Key {
    fn open() -> Result<Self, String> {
        let mut key = HKEY::default();
        // SAFETY: `key` lives through the call, which writes the opened key.
        let result = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, KEY, None, KEY_NOTIFY, &mut key) };
        result.ok().map_err(|e| format!("RegOpenKeyExW: {e}"))?;
        Ok(Self(key))
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        // SAFETY: opened by `open` and closed only here.
        let _ = unsafe { RegCloseKey(self.0) };
    }
}

/// An event that goes back to unsignalled once a wait sees it.
struct Event(HANDLE);

impl Event {
    fn new() -> Result<Self, String> {
        // SAFETY: no attributes and no name: nothing is read from memory.
        let handle = unsafe { CreateEventW(None, false, false, None) }
            .map_err(|e| format!("CreateEventW: {e}"))?;
        Ok(Self(handle))
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        // SAFETY: created by `new` and closed only here.
        let _ = unsafe { CloseHandle(self.0) };
    }
}
