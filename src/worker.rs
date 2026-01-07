use crate::{Command, CompileStage, Error, run};
use sodigy_error::{Error as SodigyError, Warning as SodigyWarning};
use sodigy_file::ModulePath;
use sodigy_span::Span;
use std::sync::mpsc;
use std::thread;

pub enum MessageToWorker {
    Run(Vec<Command>),
}

pub enum MessageToMain {
    AddModule {
        path: ModulePath,

        // def_span of the module
        span: Span,
    },
    IrComplete {
        // inter-file irs don't have `module_path`
        module_path: Option<ModulePath>,
        compile_stage: CompileStage,
        errors: Vec<SodigyError>,
        warnings: Vec<SodigyWarning>,
    },
    Error(Error),
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

    pub fn recv(&self) -> Result<MessageToMain, mpsc::RecvError> {
        self.rx_to_main.recv()
    }
}

pub fn init_workers(n: usize) -> Vec<Channel> {
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
        Err(e) => {
            tx_to_main.send(MessageToMain::Error(e)).unwrap();
        },
    });

    Channel {
        rx_to_main, tx_from_main
    }
}

fn worker_loop(
    tx_to_main: mpsc::Sender<MessageToMain>,
    rx_from_main: mpsc::Receiver<MessageToWorker>,
) -> Result<(), Error> {
    for msg in rx_from_main {
        match msg {
            MessageToWorker::Run(commands) => {
                run(commands, tx_to_main.clone())?;
            },
        }
    }

    Ok(())
}
