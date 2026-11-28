//! Programs in pseudoconsoles, with ConPTY. A thread reads the output, so that
//! the console never stops on a full pipe.

mod console;
mod output;
mod process;

use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use imperifex_domain::ports::{Pty, PtyLauncher, PtyRead, Wake};

use console::Console;
use output::{forward, Doorbell};
use process::Process;

/// How many chunks wait to be read before the program has to wait.
const WAITING_CHUNKS: usize = 64;

#[derive(Debug, Default)]
pub struct ConPtyLauncher;

struct ConPty {
    output: Receiver<Vec<u8>>,
    doorbell: Doorbell,
    input: File,
    console: Console,
    process: Process,
}

impl PtyLauncher for ConPtyLauncher {
    fn start(
        &self,
        command_line: &str,
        columns: u16,
        rows: u16,
        wake: Wake,
    ) -> Result<Box<dyn Pty>, String> {
        let (console, input, output) = Console::open(columns, rows)?;
        let (send, receive) = sync_channel(WAITING_CHUNKS);
        let doorbell = Doorbell::new(Arc::clone(&wake));
        let ringing = doorbell.clone();
        // Reading starts before the program: a full pipe would stop the console.
        std::thread::Builder::new()
            .name("conpty-output".into())
            .spawn(move || forward(output, send, &ringing))
            .map_err(|e| format!("no se pudo crear el hilo de lectura: {e}"))?;
        let process = console.launch(command_line)?;
        process.tell_when_it_leaves(wake)?;
        Ok(Box::new(ConPty {
            output: receive,
            doorbell,
            input,
            console,
            process,
        }))
    }
}

impl Pty for ConPty {
    fn read(&mut self, wait: Duration) -> PtyRead {
        self.doorbell.answer();
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
