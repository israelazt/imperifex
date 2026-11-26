//! Which processes are a terminal's, for what it costs to have it.

use std::collections::{HashMap, HashSet};

use crate::process::ProcessEntry;
use crate::profile::TerminalProfile;

/// The processes with one of the terminal's own images, and those with a helper
/// image that descend from one of them. Shells and what they run do not count:
/// the footprint is the terminal's. Images are compared ignoring case.
pub fn terminal_processes(entries: &[ProcessEntry], profile: &TerminalProfile) -> Vec<u32> {
    let is_one_of =
        |image: &str, list: &[String]| list.iter().any(|i| i.eq_ignore_ascii_case(image));
    let by_pid: HashMap<u32, &ProcessEntry> = entries.iter().map(|e| (e.pid, e)).collect();
    let own: HashSet<u32> = entries
        .iter()
        .filter(|e| is_one_of(&e.image, &profile.own_images))
        .map(|e| e.pid)
        .collect();
    entries
        .iter()
        .filter(|e| {
            own.contains(&e.pid)
                || (is_one_of(&e.image, &profile.helper_images) && descends(e, &own, &by_pid))
        })
        .map(|e| e.pid)
        .collect()
}

/// The processes of `now` that were not in `before`.
pub fn new_since(before: &[u32], now: &[u32]) -> Vec<u32> {
    let known: HashSet<&u32> = before.iter().collect();
    now.iter().copied().filter(|p| !known.contains(p)).collect()
}

/// Whether some ancestor is in `roots`. Reused pids can make the parents go in a
/// circle, so no process is visited twice.
fn descends(
    entry: &ProcessEntry,
    roots: &HashSet<u32>,
    by_pid: &HashMap<u32, &ProcessEntry>,
) -> bool {
    let mut visited = HashSet::from([entry.pid]);
    let mut parent = entry.parent;
    while visited.insert(parent) {
        if roots.contains(&parent) {
            return true;
        }
        match by_pid.get(&parent) {
            Some(next) => parent = next.parent,
            None => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(pid: u32, parent: u32, image: &str) -> ProcessEntry {
        ProcessEntry {
            pid,
            parent,
            image: image.into(),
        }
    }

    fn warp() -> TerminalProfile {
        TerminalProfile {
            name: "warp".into(),
            exe: "warp.exe".into(),
            new_window_args: vec![],
            own_images: vec!["warp.exe".into()],
            helper_images: vec!["OpenConsole.exe".into(), "conhost.exe".into()],
            new_tab_keys: None,
        }
    }

    #[test]
    fn the_terminal_and_its_helpers_count_but_not_the_shells_nor_strangers() {
        let entries = [
            entry(10, 1, "Warp.EXE"),
            entry(11, 10, "OpenConsole.exe"),
            entry(12, 11, "pwsh.exe"),
            entry(20, 1, "OpenConsole.exe"),
        ];
        assert_eq!(terminal_processes(&entries, &warp()), [10, 11]);
    }

    #[test]
    fn the_new_processes_are_those_that_were_not_there() {
        assert_eq!(new_since(&[1, 2], &[2, 3, 4]), [3, 4]);
        assert!(new_since(&[1, 2], &[1]).is_empty());
    }

    #[test]
    fn a_helper_counts_however_far_down_it_descends() {
        let entries = [
            entry(10, 1, "warp.exe"),
            entry(12, 10, "powershell.exe"),
            entry(13, 12, "conhost.exe"),
        ];
        assert_eq!(terminal_processes(&entries, &warp()), [10, 13]);
    }

    #[test]
    fn a_helper_whose_parents_go_in_a_circle_does_not_hang_nor_count() {
        let entries = [
            entry(5, 6, "conhost.exe"),
            entry(6, 5, "pwsh.exe"),
            entry(7, 7, "conhost.exe"),
        ];
        assert!(terminal_processes(&entries, &warp()).is_empty());
    }
}
