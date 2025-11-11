use crate::{Command, Error, run};
use sodigy_file::ModulePath;
use sodigy_span::Span;
use std::sync::mpsc;
use std::thread;

pub enum MessageToWorker {
    Run {
        commands: Vec<Command>,
        id: usize,
    },
}

pub enum MessageToMain {
    FoundModuleDef {
        path: ModulePath,

        // def_span of the module
        span: Span,
    },
    RunComplete {
        id: usize,
    },
    Error {
        id: usize,
        e: Error,
    },
}

pub struct Channel {
    tx_from_main: mpsc::Sender<MessageToWorker>,
    rx_to_main: mpsc::Receiver<MessageToMain>,
}

impl Channel {
    pub fn send(&self, msg: MessageToWorker) -> Result<(), mpsc::SendError<MessageToWorker>> {
        self.tx_from_main.send(msg)
    }

    pub fn try_recv(&self) -> Result<MessageToMain, mpsc::TryRecvError> {
        self.rx_to_main.try_recv()
    }
}

pub fn init_workers(n: u32) -> Vec<Channel> {
    (0..n).map(|_| init_worker()).collect()
}

fn init_worker() -> Channel {
    let (tx_to_main, rx_to_main) = mpsc::channel();
    let (tx_from_main, rx_from_main) = mpsc::channel();

    thread::spawn(move || match worker_loop(
        tx_to_main.clone(),
        rx_from_main,
    ) {
        Ok(()) => {},
        Err((id, e)) => {
            tx_to_main.send(MessageToMain::Error { id, e }).unwrap();
        },
    });

    Channel {
        rx_to_main, tx_from_main
    }
}

fn worker_loop(
    tx_to_main: mpsc::Sender<MessageToMain>,
    rx_from_main: mpsc::Receiver<MessageToWorker>,
) -> Result<(), (usize, Error)> {
    for msg in rx_from_main {
        match msg {
            MessageToWorker::Run { commands, id } => {
                run(commands, tx_to_main.clone()).map_err(|e| (id, e))?;
                tx_to_main.send(MessageToMain::RunComplete { id }).map_err(|e| (id, e.into()))?;
            },
        }
    }

    Ok(())
}
