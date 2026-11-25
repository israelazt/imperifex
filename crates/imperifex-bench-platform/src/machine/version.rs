//! The product version a program's file declares.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};

use crate::decode::{language_key, utf16, words};

/// The largest version resource accepted.
const MAX_BYTES: u32 = 1024 * 1024;

fn wide(text: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain([0]).collect()
}

/// The whole version resource of a file.
fn resource(exe: &Path) -> Result<Vec<u8>, String> {
    let path = wide(exe);
    // SAFETY: the path ends in a zero and lives through the call.
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None) };
    if size == 0 || size > MAX_BYTES {
        return Err(format!("{} no declara una versión", exe.display()));
    }
    let mut block = vec![0u8; size as usize];
    // SAFETY: the buffer is as long as the size passed.
    unsafe { GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, block.as_mut_ptr().cast()) }
        .map_err(|error| format!("no se pudo leer la versión de {}: {error}", exe.display()))?;
    Ok(block)
}

/// A value inside the resource. `unit` is the bytes of each element the system
/// counts in its answer: 2 for text, 1 for binary values.
fn value<'a>(block: &'a [u8], name: &str, unit: usize) -> Option<&'a [u8]> {
    let name = wide(name);
    let mut start = std::ptr::null_mut();
    let mut count = 0u32;
    // SAFETY: the block came from GetFileVersionInfoW, the name ends in a zero
    // and the out pointers are to locals.
    let found = unsafe {
        VerQueryValueW(
            block.as_ptr().cast(),
            PCWSTR(name.as_ptr()),
            &mut start,
            &mut count,
        )
    };
    if !found.as_bool() {
        return None;
    }
    // The pointer is only used as an offset, checked against the block.
    let from = (start as usize).checked_sub(block.as_ptr() as usize)?;
    block.get(from..from.checked_add(count as usize * unit)?)
}

pub(super) fn product_version(exe: &Path) -> Result<String, String> {
    let block = resource(exe)?;
    value(&block, r"\VarFileInfo\Translation", 1)
        .and_then(language_key)
        .and_then(|language| {
            let name = format!(r"\StringFileInfo\{language}\ProductVersion");
            value(&block, &name, 2)
        })
        .map(|bytes| utf16(&words(bytes)))
        .filter(|version| !version.is_empty())
        .ok_or_else(|| format!("{} no declara la versión del producto", exe.display()))
}
