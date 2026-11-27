//! Filling the history and cutting it again, with their times and memory.

use std::time::Instant;

use imperifex_domain::interpret::apply;
use imperifex_domain::ports::SequenceParser;
use imperifex_domain::terminal::Terminal;
use imperifex_platform::VteParser;

use crate::memory;

const LINES: usize = 1_000_000;
const CHUNK: usize = 4096;

/// 40 characters, no style.
fn plain(n: usize) -> String {
    format!("{n:07} {}\r\n", &"abcdefgh ".repeat(4)[..32])
}

/// 86 characters in four colours: two rows of 80.
fn colour(n: usize) -> String {
    let part = &"abcdefghij klmnopqr "[..18];
    format!("\x1b[31m{n:07}  {part}\x1b[32m  {part}\x1b[1;33m  {part}\x1b[0m {part}\r\n")
}

fn mib(count: usize) -> f64 {
    count as f64 / (1024.0 * 1024.0)
}

/// Feeds the lines in chunks, as a pipe gives them, and gives the bytes fed.
fn fill(terminal: &mut Terminal, line: fn(usize) -> String) -> usize {
    let mut parser = VteParser::new();
    let (mut bytes, mut fed) = (Vec::new(), 0);
    // The rows of the screen push the last lines into the history.
    for n in 0..LINES + 25 {
        bytes.extend_from_slice(line(n).as_bytes());
        if bytes.len() >= CHUNK || n == LINES + 24 {
            parser.parse(&bytes, &mut |sequence| apply(terminal, sequence));
            fed += bytes.len();
            bytes.clear();
        }
    }
    fed
}

pub fn run(case: &str) -> Result<(), String> {
    let line = match case {
        "plain" => plain,
        "colour" => colour,
        other => return Err(format!("caso desconocido: «{other}» (plain o colour)")),
    };
    let before = memory::private_bytes()?;
    let extra = || memory::private_bytes().map(|now| mib(now.saturating_sub(before)));
    let mut terminal = Terminal::with_history(80, 25, LINES);
    let start = Instant::now();
    let fed = fill(&mut terminal, line);
    let seconds = start.elapsed().as_secs_f64();
    println!(
        "caso {case}: {} filas de historial",
        terminal.screen().history_len()
    );
    println!(
        "  llenar: {seconds:.2} s, {:.0} MB de entrada (incluye generarla)",
        mib(fed)
    );
    println!("  memoria privada de más: {:.0} MB", extra()?);
    for columns in [100u16, 60, 80] {
        let start = Instant::now();
        terminal.resize(columns, 25);
        println!(
            "  reajustar a {columns} columnas: {} ms, quedan {} filas, {:.0} MB",
            start.elapsed().as_millis(),
            terminal.screen().history_len(),
            extra()?
        );
    }
    println!(
        "  pico de memoria privada del proceso: {:.0} MB",
        mib(memory::peak_private_bytes()?)
    );
    Ok(())
}
