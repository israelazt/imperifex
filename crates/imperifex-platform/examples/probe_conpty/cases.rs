//! The cases of the probe, each with what it saw.

use std::time::{Duration, Instant};

use crate::pty::Pty;

const LIMIT: Duration = Duration::from_secs(20);
const QUIET: Duration = Duration::from_millis(700);
const COLUMNS: i16 = 80;
const ROWS: i16 = 25;
const POWERSHELL: &str = "powershell.exe -NoLogo -NoProfile";

/// (name, what goes between `ESC ]` and the terminator).
const SEQUENCES: [(&str, &str); 7] = [
    ("OSC 7", "7;file://localhost/C:/carpeta%20de%20prueba"),
    ("OSC 133;A", "133;A"),
    ("OSC 133;B", "133;B"),
    ("OSC 133;C", "133;C"),
    ("OSC 133;D;3", "133;D;3"),
    ("OSC 9", "9;aviso de prueba"),
    ("OSC 9;9", "9;9;C:\\carpeta"),
];

fn escaped(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .flat_map(char::escape_debug)
        .collect()
}

fn report_close(pty: &mut Pty) {
    let closed = pty.close();
    match closed.left_in {
        Some(time) => println!("  al cerrar: salió en {} ms", time.as_millis()),
        None => println!("  al cerrar: SIGUE VIVO tras 5 s, pid {}", pty.pid),
    }
    println!("  la lectura terminó: {}", closed.reading_ended);
}

/// (a) A program that writes one mark and leaves.
fn one_shot(name: &str, command: &str) -> Result<(), String> {
    println!("\n(a) {name}: escribe y sale");
    let start = Instant::now();
    let mut pty = Pty::start(command, COLUMNS, ROWS)?;
    let left = pty.has_left_within(LIMIT);
    pty.wait_quiet(QUIET, Duration::from_secs(3));
    println!(
        "  salió solo: {left}, en {} ms",
        start.elapsed().as_millis()
    );
    report_close(&mut pty);
    let output = pty.output();
    let found = String::from_utf8_lossy(&output).contains("MARCA-1");
    println!("  marca leída: {found} ({} bytes)", output.len());
    println!("  todo lo que llegó: {}", escaped(&output));
    Ok(())
}

/// (b) and (e): typing into a shell that stays open. The typed line does not
/// hold the mark whole, so only the result can.
fn interactive(name: &str, command: &str, line: &str) -> Result<(), String> {
    println!("\n(b) {name}: escribir una orden y leer su resultado");
    let mut pty = Pty::start(command, COLUMNS, ROWS)?;
    pty.wait_quiet(QUIET, LIMIT);
    println!("  al arrancar llegó: {}", escaped(&pty.output()));
    let start = Instant::now();
    pty.send(line)?;
    let found = pty.wait_for("MARCA-2", LIMIT);
    println!(
        "  resultado leído: {found}, en {} ms",
        start.elapsed().as_millis()
    );
    println!("\n(e) {name}: cerrar la consola con la shell abierta");
    report_close(&mut pty);
    Ok(())
}

/// (c) The shell is asked for its width before and after.
fn resize() -> Result<(), String> {
    println!("\n(c) cambiar el tamaño de 80 a 100 columnas");
    let mut pty = Pty::start(POWERSHELL, COLUMNS, ROWS)?;
    pty.wait_quiet(QUIET, LIMIT);
    pty.send("'ANCHO-1=' + [Console]::WindowWidth\r")?;
    println!("  antes ve 80: {}", pty.wait_for("ANCHO-1=80", LIMIT));
    pty.resize(100, ROWS)?;
    pty.wait_quiet(QUIET, LIMIT);
    pty.send("'ANCHO-2=' + [Console]::WindowWidth\r")?;
    println!("  después ve 100: {}", pty.wait_for("ANCHO-2=100", LIMIT));
    report_close(&mut pty);
    Ok(())
}

/// Runs a PowerShell script (without double quotes) and gives what came out.
fn script_output(script: &str) -> Result<Vec<u8>, String> {
    let command = format!("{POWERSHELL} -Command \"{script}\"");
    let mut pty = Pty::start(&command, COLUMNS, ROWS)?;
    if !pty.has_left_within(LIMIT) {
        println!("  el script no terminó, pid {}", pty.pid);
    }
    pty.wait_quiet(QUIET, Duration::from_secs(3));
    pty.close();
    Ok(pty.output())
}

fn position(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|at| from + at)
}

/// Where the sequence is compared with the two marks around it.
fn verdict(output: &[u8], number: usize, payload: &str, end: &str) -> String {
    let sequence = format!("\x1b]{payload}{end}");
    let (before, after) = (format!("<<{number};"), format!(";{number}>>"));
    let Some(at) = position(output, sequence.as_bytes(), 0) else {
        let other = format!("\x1b]{payload}");
        return match position(output, other.as_bytes(), 0) {
            Some(at) => format!(
                "CAMBIADA: {}",
                escaped(&output[at..(at + other.len() + 2).min(output.len())])
            ),
            None => "NO LLEGÓ".into(),
        };
    };
    let first = position(output, before.as_bytes(), 0);
    let last = position(output, after.as_bytes(), 0);
    match (first, last) {
        (Some(first), Some(last)) if first < at && at < last => "intacta y en orden".into(),
        _ => format!("intacta, FUERA DE ORDEN (antes {first:?}, ella {at}, después {last:?})"),
    }
}

/// (d) Each sequence between two marks of text, with one terminator.
fn osc(end_name: &str, end_script: &str, end: &str) -> Result<(), String> {
    println!("\n(d) secuencias OSC terminadas en {end_name}");
    let script: String = SEQUENCES
        .iter()
        .enumerate()
        .map(|(n, (_, payload))| {
            format!(
                "[Console]::Write('<<{n};' + [char]27 + ']{payload}' + {end_script} + ';{n}>>');"
            )
        })
        .collect();
    let output = script_output(&script)?;
    for (n, (name, payload)) in SEQUENCES.iter().enumerate() {
        println!("  {name}: {}", verdict(&output, n, payload, end));
    }
    println!("  todo lo que llegó: {}", escaped(&output));
    Ok(())
}

/// (d) The same while the screen scrolls: blocks need each mark to stay
/// between the lines it was written between.
fn osc_while_scrolling() -> Result<(), String> {
    const LINES: usize = 200;
    println!("\n(d) {LINES} líneas con desplazamiento, cada una entre 133;A y 133;D");
    let script = format!(
        "$e=[char]27;$b=[char]7;foreach($i in 1..{LINES}){{\
         [Console]::Write($e+']133;A;'+$i+$b);\
         [Console]::WriteLine('linea-'+$i+'-fin');\
         [Console]::Write($e+']133;D;'+$i+$b)}}"
    );
    let output = script_output(&script)?;
    let in_order = (1..=LINES)
        .filter(|i| {
            let parts = [
                format!("\x1b]133;A;{i}\x07"),
                format!("linea-{i}-fin"),
                format!("\x1b]133;D;{i}\x07"),
            ];
            let mut from = 0;
            parts.iter().all(|part| {
                position(&output, part.as_bytes(), from)
                    .map(|at| from = at + part.len())
                    .is_some()
            })
        })
        .count();
    println!("  en orden: {in_order} de {LINES} ({} bytes)", output.len());
    let sample = &output[..output.len().min(600)];
    println!("  primeros bytes: {}", escaped(sample));
    Ok(())
}

pub fn run() -> Result<(), String> {
    one_shot("cmd", "cmd.exe /d /c echo MARCA-1")?;
    one_shot(
        "Windows PowerShell",
        &format!("{POWERSHELL} -Command \"'MARCA-1'\""),
    )?;
    interactive("cmd", "cmd.exe /d", "echo MAR^CA-2\r")?;
    interactive("Windows PowerShell", POWERSHELL, "'MAR' + 'CA-2'\r")?;
    resize()?;
    osc("BEL", "[char]7", "\x07")?;
    osc("ESC \\", "[char]27 + '\\'", "\x1b\\")?;
    osc_while_scrolling()
}
