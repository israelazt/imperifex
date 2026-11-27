//! The bytes of a shell cut into sequences by `vte` (T-D11). Plain code, built
//! and tested on every system. It decides nothing about their meaning.

use imperifex_domain::ports::SequenceParser;
use imperifex_domain::sequence::Sequence;
use vte::{Params, Perform};

/// `vte` keeps no more parameters than this.
const MAX_PARAMS: usize = 32;

#[derive(Default)]
pub struct VteParser {
    inner: vte::Parser,
}

impl VteParser {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SequenceParser for VteParser {
    fn parse(&mut self, bytes: &[u8], receive: &mut dyn FnMut(Sequence<'_>)) {
        self.inner.advance(&mut Forward { receive }, bytes);
    }
}

/// Turns what `vte` finds into our sequences. Device control strings are not
/// used, so their methods stay empty.
struct Forward<'a> {
    receive: &'a mut dyn FnMut(Sequence<'_>),
}

impl Perform for Forward<'_> {
    fn print(&mut self, character: char) {
        (self.receive)(Sequence::Print(character));
    }

    fn execute(&mut self, byte: u8) {
        (self.receive)(Sequence::Control(byte));
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], cut: bool, action: char) {
        if cut {
            return;
        }
        let mut values = [0u16; MAX_PARAMS];
        let mut length = 0;
        for (slot, param) in values.iter_mut().zip(params.iter()) {
            *slot = param.first().copied().unwrap_or(0);
            length += 1;
        }
        (self.receive)(Sequence::Csi {
            params: &values[..length],
            intermediates,
            action,
        });
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], cut: bool, action: u8) {
        if !cut {
            (self.receive)(Sequence::Esc {
                intermediates,
                action,
            });
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        (self.receive)(Sequence::Osc(params));
    }
}

#[cfg(test)]
mod tests;
