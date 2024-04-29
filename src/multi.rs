use crate::{
    construct_hir,
    PathOrRawInput,
};
use log::info;
use sodigy_ast::IdentWithSpan;
use sodigy_config::CompilerOption;
use sodigy_high_ir::get_global_hir_cache;
use std::sync::mpsc;
use std::thread;

type Path = String;

pub enum MessageFromMain {
    ConstructHirSession { name: String, path: Path },
    YouShouldAskForAJob,
    KillImmd,
}

pub enum MessageToMain {
    HirComplete { imported_names: Vec<IdentWithSpan> },
    GiveMeAJob,
}

pub struct Channel {
    tx_from_main: mpsc::Sender<MessageFromMain>,
    rx_to_main: mpsc::Receiver<MessageToMain>,
}

impl Channel {
    pub fn send(&self, msg: MessageFromMain) -> Result<(), mpsc::SendError<MessageFromMain>> {
        self.tx_from_main.send(msg)
    }

    pub fn try_recv(&self) -> Result<MessageToMain, mpsc::TryRecvError> {
        self.rx_to_main.try_recv()
    }

    pub fn block_recv(&self) -> Result<MessageToMain, mpsc::RecvError> {
        self.rx_to_main.recv()
    }
}

pub fn kill_all_workers(channels: &Vec<Channel>) {
    for worker in channels.iter() {
        // it doesn't have to unwrap
        let _ = worker.send(MessageFromMain::KillImmd);
    }
}

pub fn init_hir_workers(n: usize, compiler_option: CompilerOption) -> Vec<Channel> {
    (0..n).map(|i| init_hir_worker(i, compiler_option.clone())).collect()
}

pub fn init_hir_worker(index: usize, compiler_option: CompilerOption) -> Channel {
    let (tx_to_main, rx_to_main) = mpsc::channel();
    let (tx_from_main, rx_from_main) = mpsc::channel();

    thread::spawn(move || {
        event_loop(tx_to_main, rx_from_main, index, compiler_option);
    });

    Channel {
        rx_to_main, tx_from_main
    }
}

pub fn distribute_messages(
    messages: Vec<MessageFromMain>,
    channels: &[Channel],
) -> Result<(), mpsc::SendError<MessageFromMain>> {
    for (index, message) in messages.into_iter().enumerate() {
        channels[index % channels.len()].send(message)?;
    }

    Ok(())
}

pub fn event_loop(
    tx_to_main: mpsc::Sender<MessageToMain>,
    rx_from_main: mpsc::Receiver<MessageFromMain>,
    index: usize,
    compiler_option: CompilerOption,
) {
    info!("worker [{index}] initialized!");
    let global_hir_cache = unsafe { get_global_hir_cache() };

    for msg in rx_from_main {
        match msg {
            MessageFromMain::ConstructHirSession { name, path } => {
                info!("worker [{index}] got message: ConstructHirSession({path:?})");

                let result = construct_hir(
                    PathOrRawInput::Path(&path),
                    &compiler_option,

                    // workers never handle a root file
                    false,  // is_root
                );

                let imported_names = match &result {
                    (Some(session), _) => session.imported_names.clone(),
                    _ => vec![],
                };

                global_hir_cache.push_result(
                    name,
                    result,
                );

                tx_to_main.send(
                    MessageToMain::HirComplete { imported_names },
                ).unwrap();
            },
            MessageFromMain::YouShouldAskForAJob => {
                tx_to_main.send(
                    MessageToMain::GiveMeAJob,
                ).unwrap();
            },
            MessageFromMain::KillImmd => {
                return;
            },
        }
    }

    drop(tx_to_main)
}
