//! The echo program: it writes back each key as soon as it arrives.

use imperifex_bench_domain::echo::{echo_reply, ECHO_PROMPT};
use imperifex_bench_domain::ports::{KeyInput, Output};

/// Enough for any burst of keys; each read answers with what there is.
const BUFFER: usize = 256;

fn say(out: &mut dyn Output, bytes: &[u8]) -> Result<(), String> {
    out.write_all(bytes)?;
    // Every key at once: this is what is being timed.
    out.flush()
}

/// Runs until `q` is typed, or the input ends.
pub fn run_echo(input: &mut dyn KeyInput, out: &mut dyn Output) -> Result<(), String> {
    say(out, ECHO_PROMPT.as_bytes())?;
    let mut buffer = [0; BUFFER];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            return Ok(());
        }
        let reply = echo_reply(&buffer[..read]);
        if !reply.bytes.is_empty() {
            say(out, &reply.bytes)?;
        }
        if reply.quit {
            return say(out, b"\r\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Event, Events, FakeKeys, FakeOutput};

    fn run(reads: &[&[u8]]) -> Vec<Event> {
        let events = Events::default();
        let mut input = FakeKeys::new(reads);
        run_echo(&mut input, &mut FakeOutput::new(&events)).unwrap();
        let seen = events.borrow().clone();
        seen
    }

    fn wrote(bytes: &[u8]) -> Event {
        Event::Wrote(bytes.to_vec())
    }

    #[test]
    fn each_key_is_written_and_flushed_as_it_arrives() {
        let events = run(&[b"a", b"b", b"q"]);
        assert_eq!(
            events,
            [
                wrote(ECHO_PROMPT.as_bytes()),
                Event::Flushed,
                wrote(b"a"),
                Event::Flushed,
                wrote(b"b"),
                Event::Flushed,
                wrote(b"\r\n"),
                Event::Flushed,
            ]
        );
    }

    #[test]
    fn what_is_ignored_writes_nothing() {
        assert_eq!(run(&[b"\x1b", b"q"]).len(), 4);
    }

    #[test]
    fn the_end_of_the_input_ends_it_too() {
        assert_eq!(run(&[]).len(), 2);
    }
}
