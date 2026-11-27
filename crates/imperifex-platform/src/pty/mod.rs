//! Programs in pseudoconsoles, with ConPTY. A thread reads the output, so that
//! the console never stops on a full pipe.

mod console;
mod process;

use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;

use imperifex_domain::ports::{Pty, PtyLauncher, PtyRead};

use console::Console;
use process::Process;

/// How many chunks wait to be read before the program has to wait.
const WAITING_CHUNKS: usize = 64;
const CHUNK: usize = 16 * 1024;

#[derive(Debug, Default)]
pub struct ConPtyLauncher;

struct ConPty {
    output: Receiver<Vec<u8>>,
    input: File,
    console: Console,
    process: Process,
}

impl PtyLauncher for ConPtyLauncher {
    fn start(&self, command_line: &str, columns: u16, rows: u16) -> Result<Box<dyn Pty>, String> {
        let (console, input, output) = Console::open(columns, rows)?;
        let (send, receive) = sync_channel(WAITING_CHUNKS);
        // Reading starts before the program: a full pipe would stop the console.
        std::thread::Builder::new()
            .name("conpty-output".into())
            .spawn(move || forward(output, &send))
            .map_err(|e| format!("no se pudo crear el hilo de lectura: {e}"))?;
        let process = console.launch(command_line)?;
        Ok(Box::new(ConPty {
            output: receive,
            input,
            console,
            process,
        }))
    }
}

/// Passes the output on until the console closes its pipe. With nobody left to
/// take it, it is still read and dropped: closing may wait for that.
fn forward(mut from: File, to: &SyncSender<Vec<u8>>) {
    let mut chunk = vec![0u8; CHUNK];
    let mut wanted = true;
    // An error here is the pipe closing: the end of the output.
    while let Ok(read @ 1..) = from.read(&mut chunk) {
        wanted = wanted && to.send(chunk[..read].to_vec()).is_ok();
    }
}

impl Pty for ConPty {
    fn read(&mut self, wait: Duration) -> PtyRead {
        match self.output.recv_timeout(wait) {
            Ok(bytes) => PtyRead::Data(bytes),
            Err(RecvTimeoutError::Timeout) => PtyRead::Nothing,
            Err(RecvTimeoutError::Disconnected) => PtyRead::End,
        }
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.input
            .write_all(bytes)
            .map_err(|e| format!("no se pudo escribir en la consola: {e}"))
    }

    fn resize(&mut self, columns: u16, rows: u16) -> Result<(), String> {
        self.console.resize(columns, rows)
    }

    fn exit_code(&mut self) -> Result<Option<u32>, String> {
        self.process.exit_code()
    }

    fn close(&mut self) {
        self.console.close();
    }

    fn process_id(&self) -> u32 {
        self.process.id()
    }
}
