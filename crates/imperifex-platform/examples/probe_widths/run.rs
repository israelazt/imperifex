//! A PowerShell script writes each sample and then the column the console says
//! it ended in; the same sample goes into a grid of our own.

use std::time::{Duration, Instant};

use imperifex_domain::grid::Grid;
use imperifex_domain::interpret::apply;
use imperifex_domain::ports::{PtyLauncher, PtyRead, SequenceParser};
use imperifex_domain::terminal::Terminal;
use imperifex_platform::{ConPtyLauncher, VteParser};

const SAMPLES: [(&str, &str); 10] = [
    ("latina con acento suelto", "e\u{301}"),
    ("china", "漢字"),
    ("emoji", "🚀"),
    ("familia unida con ZWJ", "👨\u{200d}👩\u{200d}👧"),
    ("corazón con FE0F", "❤\u{fe0f}"),
    ("corazón sin FE0F", "❤"),
    ("bandera", "🇲🇽"),
    ("pulgar con tono", "👍\u{1f3fd}"),
    ("tecla 1", "1\u{fe0f}\u{20e3}"),
    ("ambiguos", "→±…│"),
];

/// The script writes each sample from its code points, so that nothing depends
/// on how the command line carries them.
fn script() -> String {
    let mut script = String::from("[Console]::OutputEncoding=[Text.Encoding]::UTF8;");
    for (number, (_, sample)) in SAMPLES.iter().enumerate() {
        let codes: Vec<String> = sample.chars().map(|c| u32::from(c).to_string()).collect();
        script.push_str(&format!(
            "$s=-join(@({})|%{{[char]::ConvertFromUtf32($_)}});[Console]::Write($s);\
             $c=[Console]::CursorLeft;[Console]::WriteLine(' [{number}:'+$c+']');",
            codes.join(",")
        ));
    }
    script.push_str("$e=[char]27;[Console]::WriteLine('dibujo '+$e+'(0lqqk'+$e+'(B fin')");
    script
}

/// The screen after running the script.
fn screen() -> Result<(Terminal, Vec<u8>), String> {
    let command = format!(
        "powershell.exe -NoLogo -NoProfile -Command \"{}\"",
        script()
    );
    let mut pty = ConPtyLauncher.start(&command, 80, 25)?;
    let mut terminal = Terminal::new(80, 25);
    let mut parser = VteParser::new();
    let mut raw = Vec::new();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(20) {
        match pty.read(Duration::from_millis(50)) {
            PtyRead::Data(bytes) => {
                parser.parse(&bytes, &mut |sequence| apply(&mut terminal, sequence));
                raw.extend(bytes);
            }
            PtyRead::Nothing => {
                if pty.exit_code()?.is_some() {
                    pty.close();
                }
            }
            PtyRead::End => return Ok((terminal, raw)),
        }
    }
    Err("la salida no terminó en 20 s".into())
}

fn our_columns(sample: &str) -> usize {
    let mut grid = Grid::new(80, 1);
    for character in sample.chars() {
        grid.print(character);
    }
    grid.cursor().column
}

pub fn run() -> Result<(), String> {
    let (terminal, raw) = screen()?;
    let text = terminal.screen().text();
    println!("caso | consola | nuestra cuadrícula");
    for (number, (name, sample)) in SAMPLES.iter().enumerate() {
        let mark = format!("[{number}:");
        let console = text
            .split_once(&mark)
            .and_then(|(_, rest)| rest.split_once(']'))
            .map_or("?", |(column, _)| column);
        let ours = our_columns(sample);
        let same = if console == ours.to_string() {
            ""
        } else {
            "  <-- DISTINTO"
        };
        println!("{name} | {console} | {ours}{same}");
    }
    println!("\nnuestra pantalla:\n{}", text.trim_end());
    let raw = String::from_utf8_lossy(&raw);
    let drawing = raw.find("dibujo").map_or("no llegó", |at| &raw[at..]);
    println!("\nbytes del dibujo: {}", drawing.escape_debug());
    Ok(())
}
