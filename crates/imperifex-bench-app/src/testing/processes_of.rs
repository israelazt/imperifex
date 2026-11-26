//! The processes that go with a fake desktop: the author's and the ones its
//! launches started. All are `warp.exe` unless renamed.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use imperifex_bench_domain::ports::ProcessProbe;
use imperifex_bench_domain::process::{ProcessEntry, ProcessUsage};

use super::desktop::{FakeDesktop, State};

#[derive(Debug, Clone)]
pub struct DesktopProcesses {
    author: Vec<u32>,
    state: Rc<RefCell<State>>,
    images: Rc<RefCell<HashMap<u32, String>>>,
    calls: Rc<Cell<usize>>,
}

impl DesktopProcesses {
    pub fn new(desktop: &FakeDesktop, author: &[u32]) -> Self {
        Self {
            author: author.to_vec(),
            state: Rc::clone(&desktop.state),
            images: Rc::default(),
            calls: Rc::default(),
        }
    }

    pub fn rename(&self, pid: u32, image: &str) {
        self.images.borrow_mut().insert(pid, image.into());
    }

    /// How many times the list was asked for.
    pub fn calls(&self) -> usize {
        self.calls.get()
    }
}

/// One look has gone by for the processes on their way out.
fn age(state: &mut State) {
    let leaving = std::mem::take(&mut state.leaving);
    for (looks, pid) in leaving {
        match looks.checked_sub(1) {
            Some(left) => state.leaving.push((left, pid)),
            None => state.alive.retain(|&p| p != pid),
        }
    }
}

impl ProcessProbe for DesktopProcesses {
    fn processes(&self) -> Result<Vec<ProcessEntry>, String> {
        self.calls.set(self.calls.get() + 1);
        let mut state = self.state.borrow_mut();
        age(&mut state);
        let images = self.images.borrow();
        Ok(self
            .author
            .iter()
            .chain(&state.alive)
            .map(|&pid| ProcessEntry {
                pid,
                parent: 0,
                image: images.get(&pid).map_or("warp.exe", String::as_str).into(),
            })
            .collect())
    }

    fn usage(&self, _pid: u32) -> Result<ProcessUsage, String> {
        Ok(ProcessUsage::default())
    }
}
