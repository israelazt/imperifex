//! Sends a load to the terminal the program is running in, and times how long
//! the terminal takes to accept it.

use std::path::Path;

use imperifex_bench_domain::emit::EmitResult;
use imperifex_bench_domain::loads::generate;
use imperifex_bench_domain::metric::Load;
use imperifex_bench_domain::ports::{FileStore, Output};
use imperifex_domain::ports::Clock;

/// The size of a load when none is asked for. ASCII is the fastest to take in,
/// so it needs more to be timed well.
pub fn default_mib(load: Load) -> usize {
    if load == Load::Ascii {
        100
    } else {
        32
    }
}

/// The size of the pieces the load is handed over in.
const CHUNK: usize = 64 * 1024;

pub struct EmitService<C> {
    clock: C,
}

impl<C: Clock> EmitService<C> {
    pub fn new(clock: C) -> Self {
        Self { clock }
    }

    /// The load is made before the clock starts. Afterwards, outside the time
    /// measured, the colours are reset and a last line says it is over.
    pub fn run(
        &self,
        out: &mut dyn Output,
        load: Load,
        bytes: usize,
        seed: u64,
    ) -> Result<EmitResult, String> {
        let data = generate(load, bytes, seed);
        let start = self.clock.monotonic();
        for chunk in data.chunks(CHUNK) {
            out.write_all(chunk)?;
        }
        out.flush()?;
        let took = self.clock.monotonic().saturating_sub(start);
        let done = format!("\x1b[0m\r\nIMPERIFEX-BENCH-DONE {}\r\n", load.name());
        out.write_all(done.as_bytes())?;
        out.flush()?;
        Ok(EmitResult::new(load, bytes, took))
    }

    /// Keeps a result as JSON.
    pub fn save(files: &dyn FileStore, path: &Path, result: &EmitResult) -> Result<(), String> {
        files.write(path, result.to_json().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Event, Events, FakeClock, FakeFiles, FakeOutput};

    const BYTES: usize = 2 * CHUNK + 10;

    fn emit() -> (EmitResult, Vec<Event>) {
        let events = Events::default();
        let service = EmitService::new(FakeClock::ticking(500, &events));
        let mut out = FakeOutput::new(&events);
        let result = service.run(&mut out, Load::Ascii, BYTES, 1).unwrap();
        let seen = events.borrow().clone();
        (result, seen)
    }

    fn written(events: &[Event]) -> Vec<&[u8]> {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Wrote(bytes) => Some(bytes.as_slice()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_load_goes_in_pieces_of_64_kib_but_the_last() {
        let (_, events) = emit();
        let sizes: Vec<usize> = written(&events).iter().map(|w| w.len()).collect();
        assert_eq!(sizes[..3], [CHUNK, CHUNK, 10]);
        assert_eq!(
            written(&events)[..3].concat(),
            generate(Load::Ascii, BYTES, 1)
        );
    }

    #[test]
    fn the_time_is_the_clocks_and_only_the_load_is_inside_it() {
        let (result, events) = emit();
        assert_eq!((result.seconds, result.bytes), (0.5, BYTES));
        let shape: Vec<&str> = events
            .iter()
            .map(|event| match event {
                Event::ClockRead => "clock",
                Event::Wrote(_) => "write",
                Event::Flushed => "flush",
            })
            .collect();
        assert_eq!(
            shape,
            ["clock", "write", "write", "write", "flush", "clock", "write", "flush"]
        );
    }

    #[test]
    fn the_last_thing_written_resets_the_colours_and_says_it_is_over() {
        let (_, events) = emit();
        assert_eq!(
            *written(&events).last().unwrap(),
            b"\x1b[0m\r\nIMPERIFEX-BENCH-DONE ascii\r\n"
        );
    }

    #[test]
    fn an_output_that_fails_is_an_error_and_nothing_else_is_written() {
        let events = Events::default();
        let service = EmitService::new(FakeClock::ticking(1, &events));
        let mut out = FakeOutput::failing(&events, "cerrada");
        let error = service.run(&mut out, Load::Ascii, BYTES, 1).unwrap_err();
        assert_eq!(error, "cerrada");
        assert_eq!(written(&events.borrow()).len(), 1);
    }

    #[test]
    fn a_result_is_saved_as_its_json() {
        let (result, _) = emit();
        let files = FakeFiles::default();
        EmitService::<FakeClock>::save(&files, Path::new("a.json"), &result).unwrap();
        let saved = files.read(Path::new("a.json")).unwrap().unwrap();
        assert_eq!(saved, result.to_json().into_bytes());
    }
}
