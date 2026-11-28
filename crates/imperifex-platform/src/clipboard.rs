//! The text of the Windows clipboard.

use imperifex_domain::ports::Clipboard;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};

/// The format of text in UTF-16, ended by a zero.
const UNICODE_TEXT: u32 = 13;

/// More than this is not taken from the clipboard: 8 million characters.
const MOST_UNITS: usize = 8 * 1024 * 1024;

pub struct WindowsClipboard;

/// The clipboard while it is ours: others wait until it is closed.
struct Open;

impl Open {
    fn new() -> Result<Self, String> {
        // SAFETY: with no window the clipboard belongs to this task.
        unsafe { OpenClipboard(None) }.map_err(|e| format!("OpenClipboard: {e}"))?;
        Ok(Self)
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        // SAFETY: only an `Open` that opened it is ever dropped.
        if let Err(error) = unsafe { CloseClipboard() } {
            imperifex_domain::log_warn!("CloseClipboard: {error}");
        }
    }
}

/// The memory of `block` while it is held in place.
struct Locked {
    block: HGLOBAL,
    at: *mut u16,
}

impl Locked {
    fn new(block: HGLOBAL) -> Result<Self, String> {
        // SAFETY: a block of global memory that is alive.
        let at = unsafe { GlobalLock(block) }.cast::<u16>();
        if at.is_null() {
            return Err("GlobalLock: no se pudo fijar la memoria".into());
        }
        Ok(Self { block, at })
    }
}

impl Drop for Locked {
    fn drop(&mut self) {
        // SAFETY: locked once by `new`. It answers an error when the count
        // reaches zero, which is what is wanted: there is nothing to do.
        let _ = unsafe { GlobalUnlock(self.block) };
    }
}

fn read_block(block: HGLOBAL) -> Result<String, String> {
    // SAFETY: a block of global memory that the clipboard holds.
    let bytes = unsafe { GlobalSize(block) };
    let units = (bytes / 2).min(MOST_UNITS);
    let locked = Locked::new(block)?;
    // SAFETY: the block has at least `units` units and stays in place while
    // `locked` lives.
    let text = unsafe { std::slice::from_raw_parts(locked.at, units) };
    let end = text.iter().position(|&unit| unit == 0).unwrap_or(units);
    Ok(String::from_utf16_lossy(&text[..end]))
}

impl Clipboard for WindowsClipboard {
    fn read(&self) -> Result<Option<String>, String> {
        let _open = Open::new()?;
        // SAFETY: the clipboard is open. With no text in it, it is an error.
        let Ok(handle) = (unsafe { GetClipboardData(UNICODE_TEXT) }) else {
            return Ok(None);
        };
        read_block(HGLOBAL(handle.0)).map(Some)
    }

    fn write(&self, text: &str) -> Result<(), String> {
        let units: Vec<u16> = text.encode_utf16().chain([0]).collect();
        let _open = Open::new()?;
        // SAFETY: the clipboard is open.
        unsafe { EmptyClipboard() }.map_err(|e| format!("EmptyClipboard: {e}"))?;
        // SAFETY: a new block of that many bytes.
        let block = unsafe { GlobalAlloc(GMEM_MOVEABLE, units.len() * 2) }
            .map_err(|e| format!("GlobalAlloc: {e}"))?;
        let given = fill(block, &units).and_then(|()| give(block));
        if given.is_err() {
            // SAFETY: the clipboard did not take it: it is still ours.
            if let Err(error) = unsafe { GlobalFree(Some(block)) } {
                imperifex_domain::log_warn!("GlobalFree: {error}");
            }
        }
        given
    }
}

fn fill(block: HGLOBAL, units: &[u16]) -> Result<(), String> {
    let locked = Locked::new(block)?;
    // SAFETY: the block was made with room for these units.
    unsafe { std::ptr::copy_nonoverlapping(units.as_ptr(), locked.at, units.len()) };
    Ok(())
}

/// Hands `block` to the clipboard, which owns it from then on.
fn give(block: HGLOBAL) -> Result<(), String> {
    // SAFETY: the clipboard is open and the block holds text ended by zero.
    unsafe { SetClipboardData(UNICODE_TEXT, Some(HANDLE(block.0))) }
        .map(drop)
        .map_err(|e| format!("SetClipboardData: {e}"))
}
