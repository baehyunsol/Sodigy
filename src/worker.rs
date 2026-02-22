use crate::TimingsEntry;
use sodigy::{
    Command,
    EmitIrOption,
    Error,
    StoreIrAt,
    emit_irs_if_has_to,
    get_cached_ir,
};
use sodigy_driver::CompileStage;
use sodigy_endec::Endec;
use sodigy_error::{Error as SodigyError, Warning as SodigyWarning};
use sodigy_file::{File, FileOrStd, ModulePath};
use sodigy_fs_api::{WriteMode, join3, read_bytes, write_bytes};
use sodigy_hir as hir;
use sodigy_mir as mir;
use sodigy_span::Span;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkerId(pub usize);

pub enum MessageToWorker {
    Run(Vec<Command>),
    Kill,
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
    TimingsLog {
        worker_id: WorkerId,
        entries: Vec<TimingsEntry>,
    },
    Error(Error),
}

pub struct Channel {
    pub worker_id: WorkerId,
    tx_from_main: mpsc::Sender<MessageToWorker>,
    rx_to_main: mpsc::Receiver<MessageToMain>,
    join_handle: JoinHandle<()>,
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

    /// It tries to collect logs from the worker, then joins the thread.
    /// If it cannot collect the logs (timeout = 500ms), it returns `None`.
    /// The result of `join_handle.join()` is always ignored.
    pub fn join(self) -> Option<Vec<TimingsEntry>> {
        let started_at = Instant::now();
        let log = loop {
            match self.try_recv() {
                Ok(MessageToMain::TimingsLog { entries, .. }) => {
                    break Some(entries);
                },
                Ok(_) | Err(mpsc::TryRecvError::Empty) => {},
                _ => {
                    break None;
                },
            }

            if Instant::now().duration_since(started_at.clone()).as_millis() > 500 {
                break None;
            }
        };

        let _ = self.join_handle.join();
        log
    }
}

pub fn init_workers_and_channels(n: usize) -> Vec<Channel> {
    (0..n).map(|i| init_worker_and_channel(i)).collect()
}

fn init_worker_and_channel(id: usize) -> Channel {
    let worker_id = WorkerId(id);
    let (tx_to_main, rx_to_main) = mpsc::channel();
    let (tx_from_main, rx_from_main) = mpsc::channel();

    let join_handle = thread::Builder::new()
        .name(format!("worker-{id}"))
        .spawn(move || match worker_loop(tx_to_main.clone(), rx_from_main, worker_id) {
            Ok(()) => {},
            Err(e) => {
                tx_to_main.send(MessageToMain::Error(e)).unwrap();
            },
        }).unwrap();

    Channel {
        worker_id,
        rx_to_main,
        tx_from_main,
        join_handle,
    }
}

/// The main thread owns `Channel`, and each worker thread
/// owns `Worker`. `Worker` is a very thin wrapper. Its main
/// purpose is logging.
pub struct Worker {
    pub id: WorkerId,
    pub born_at: Instant,
    pub timings_log: Vec<TimingsEntry>,
    pub log_file: Option<String>,
    pub curr_stage: Option<(CompileStage, Option<String>, u64)>,
    pub curr_stage_error: bool,
}

fn worker_loop(
    tx_to_main: mpsc::Sender<MessageToMain>,
    rx_from_main: mpsc::Receiver<MessageToWorker>,
    worker_id: WorkerId,
) -> Result<(), Error> {
    let mut worker = Worker {
        id: worker_id,
        born_at: Instant::now(),
        timings_log: vec![],

        // NOTE: Currently, there's no API that sets this value.
        //       You have to hard-code the log file and re-compile it...
        // log_file: Some(String::from("log")),
        log_file: None,

        curr_stage: None,
        curr_stage_error: false,
    };

    for msg in rx_from_main {
        match msg {
            MessageToWorker::Run(commands) => {
                if let Err(e) = worker.run_commands(commands, tx_to_main.clone()) {
                    if worker.curr_stage.is_some() {
                        worker.log_end(true);
                    }

                    tx_to_main.send(MessageToMain::TimingsLog {
                        worker_id,
                        entries: worker.timings_log.drain(..).collect(),
                    })?;
                    return Err(e);
                }
            },
            MessageToWorker::Kill => {
                tx_to_main.send(MessageToMain::TimingsLog {
                    worker_id,
                    entries: worker.timings_log.drain(..).collect(),
                })?;
                break;
            },
        }
    }

    Ok(())
}

impl Worker {
    pub fn run_commands(
        &mut self,
        commands: Vec<Command>,
        tx_to_main: mpsc::Sender<MessageToMain>,
    ) -> Result<(), Error> {
        for command in commands.into_iter() {
            match command {
                Command::PerFileIr {
                    input_file_path,
                    input_module_path,
                    optimize_level,
                    intermediate_dir,
                    find_modules,
                    emit_ir_options,
                    stop_after,
                } => {
                    let (is_std, file) = match &input_file_path {
                        FileOrStd::File(path) => (
                            false,
                            File::register(
                                0,  // project_id
                                &path,
                                &input_module_path.to_string(),
                                &intermediate_dir,
                            )?,
                        ),
                        FileOrStd::Std(n) => (true, File::Std(*n)),
                    };
                    let content_hash = file.get_content_hash(&intermediate_dir)?;

                    let mut mir_session = if stop_after >= CompileStage::Mir && let Some(mir_session_bytes) = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::Mir,
                        Some(content_hash),
                    )? {
                        let mut s = mir::Session::decode(&mir_session_bytes)?;
                        s.intermediate_dir = intermediate_dir.clone();
                        s
                    } else {
                        let mut hir_session = if let Some(hir_session_bytes) = get_cached_ir(
                            &intermediate_dir,
                            CompileStage::Hir,
                            Some(content_hash),
                        )? {
                            let mut s = hir::Session::decode(&hir_session_bytes)?;
                            s.intermediate_dir = intermediate_dir.clone();
                            s
                        } else {
                            self.log_start(CompileStage::Load, Some(input_module_path.to_string()));
                            let bytes = file.read_bytes(&intermediate_dir)?.ok_or(Error::MiscError)?;
                            self.log_end(false);

                            self.log_start(CompileStage::Lex, Some(input_module_path.to_string()));
                            let lex_session = sodigy_lex::lex(
                                file,
                                bytes,
                                intermediate_dir.clone(),
                                is_std,
                            );
                            self.log_end(!lex_session.errors.is_empty());

                            emit_irs_if_has_to(
                                &lex_session,
                                &emit_ir_options,
                                CompileStage::Lex,
                                Some(content_hash),
                                &intermediate_dir,
                            )?;

                            if !lex_session.errors.is_empty() || stop_after <= CompileStage::Lex {
                                tx_to_main.send(MessageToMain::IrComplete {
                                    module_path: Some(input_module_path),
                                    compile_stage: CompileStage::Lex,
                                    errors: lex_session.errors.clone(),
                                    warnings: lex_session.warnings.clone(),
                                })?;

                                if !lex_session.errors.is_empty() {
                                    return Err(Error::CompileError);
                                }

                                continue;
                            }

                            self.log_start(CompileStage::Parse, Some(input_module_path.to_string()));
                            let parse_session = sodigy_parse::parse(lex_session);
                            self.log_end(!parse_session.errors.is_empty());

                            emit_irs_if_has_to(
                                &parse_session,
                                &emit_ir_options,
                                CompileStage::Parse,
                                Some(content_hash),
                                &intermediate_dir,
                            )?;

                            if !parse_session.errors.is_empty() || stop_after <= CompileStage::Parse {
                                tx_to_main.send(MessageToMain::IrComplete {
                                    module_path: Some(input_module_path),
                                    compile_stage: CompileStage::Parse,
                                    errors: parse_session.errors.clone(),
                                    warnings: parse_session.warnings.clone(),
                                })?;

                                if !parse_session.errors.is_empty() {
                                    return Err(Error::CompileError);
                                }

                                continue;
                            }

                            self.log_start(CompileStage::Hir, Some(input_module_path.to_string()));
                            let hir_session = sodigy_hir::lower(parse_session);
                            self.log_end(!hir_session.errors.is_empty());

                            emit_irs_if_has_to(
                                &hir_session,
                                &emit_ir_options,
                                CompileStage::Hir,
                                Some(content_hash),
                                &intermediate_dir,
                            )?;
                            hir_session
                        };

                        if find_modules {
                            for module in hir_session.modules.iter() {
                                let module_name = module.name.unintern_or_default(&intermediate_dir);
                                tx_to_main.send(MessageToMain::AddModule {
                                    path: input_module_path.join(module_name),
                                    span: module.name_span,
                                })?;
                            }
                        }

                        if !hir_session.errors.is_empty() || stop_after <= CompileStage::Hir {
                            tx_to_main.send(MessageToMain::IrComplete {
                                module_path: Some(input_module_path),
                                compile_stage: CompileStage::Hir,
                                errors: hir_session.errors.clone(),
                                warnings: hir_session.warnings.clone(),
                            })?;

                            if !hir_session.errors.is_empty() {
                                return Err(Error::CompileError);
                            }

                            continue;
                        }

                        // the inter-hir session must have been created at this point
                        let inter_hir_session_bytes = get_cached_ir(
                            &intermediate_dir,
                            CompileStage::InterHir,
                            None,
                        )?.ok_or(Error::IrCacheNotFound(CompileStage::InterHir))?;
                        let mut inter_hir_session = sodigy_inter_hir::Session::decode(&inter_hir_session_bytes)?;
                        inter_hir_session.intermediate_dir = intermediate_dir.clone();

                        // TODO: I want to log this, but there's no `CompileStage` for this.
                        let _ = inter_hir_session.resolve_module(&mut hir_session);

                        hir_session.errors.extend(inter_hir_session.errors.drain(..));
                        hir_session.warnings.extend(inter_hir_session.warnings.drain(..));

                        // inter-hir may create new funcs and poly-generics, and the new functions
                        // must belong to some module. They all go to `lib.sdg`.
                        // TODO: distribute the new functions for better parallelism
                        if input_module_path.is_lib() {
                            hir_session.funcs.extend(inter_hir_session.new_funcs.drain(..));
                            hir_session.polys.extend(inter_hir_session.new_polys.drain());
                        }

                        self.log_start(CompileStage::Mir, Some(input_module_path.to_string()));
                        let mut mir_session = sodigy_mir::lower(hir_session, &inter_hir_session);
                        self.log_end(!mir_session.errors.is_empty());

                        init_span_string_map_if_necessary(
                            &mut mir_session,
                            &emit_ir_options,
                            &intermediate_dir,
                            /* read_from_file: */ false,
                            /* write_to_file: */ false,
                        )?;
                        emit_irs_if_has_to(
                            &mir_session,
                            &emit_ir_options,
                            CompileStage::Mir,
                            Some(content_hash),
                            &intermediate_dir,
                        )?;

                        mir_session
                    };

                    if !mir_session.errors.is_empty() || stop_after <= CompileStage::Mir {
                        tx_to_main.send(MessageToMain::IrComplete {
                            module_path: Some(input_module_path),
                            compile_stage: CompileStage::Mir,
                            errors: mir_session.errors.clone(),
                            warnings: mir_session.warnings.clone(),
                        })?;

                        if !mir_session.errors.is_empty() {
                            return Err(Error::CompileError);
                        }

                        continue;
                    }

                    // the inter-mir session must have been created at this point
                    let inter_mir_session_bytes = get_cached_ir(
                        &intermediate_dir,
                        CompileStage::InterMir,
                        None,
                    )?.ok_or(Error::IrCacheNotFound(CompileStage::InterMir))?;
                    let mut inter_mir_session = sodigy_inter_mir::Session::decode(&inter_mir_session_bytes)?;
                    mir_session.errors.extend(inter_mir_session.errors.drain(..));
                    mir_session.warnings.extend(inter_mir_session.warnings.drain(..));
                    mir_session.types = inter_mir_session.types.drain().collect();
                    mir_session.generic_args = inter_mir_session.generic_args.drain().collect();

                    self.log_start(CompileStage::PostMir, Some(input_module_path.to_string()));
                    let _ = sodigy_post_mir::lower(&mut mir_session);
                    self.log_end(!mir_session.errors.is_empty());

                    init_span_string_map_if_necessary(
                        &mut mir_session,
                        &emit_ir_options,
                        &intermediate_dir,
                        /* read_from_file: */ true,
                        /* write_to_file: */ false,
                    )?;
                    emit_irs_if_has_to(
                        &mir_session,
                        &emit_ir_options,
                        CompileStage::PostMir,
                        Some(content_hash),
                        &intermediate_dir,
                    )?;

                    if !mir_session.errors.is_empty() || stop_after <= CompileStage::PostMir {
                        tx_to_main.send(MessageToMain::IrComplete {
                            module_path: Some(input_module_path),
                            compile_stage: CompileStage::PostMir,
                            errors: mir_session.errors.clone(),
                            warnings: mir_session.warnings.clone(),
                        })?;

                        if !mir_session.errors.is_empty() {
                            return Err(Error::CompileError);
                        }

                        continue;
                    }

                    self.log_start(CompileStage::MirOptimize, Some(input_module_path.to_string()));
                    let optimized_mir_session = sodigy_optimize::optimize_mir(mir_session, optimize_level);
                    self.log_end(!optimized_mir_session.errors.is_empty());

                    emit_irs_if_has_to(
                        &optimized_mir_session,
                        &emit_ir_options,
                        CompileStage::MirOptimize,
                        Some(content_hash),
                        &intermediate_dir,
                    )?;

                    if !optimized_mir_session.errors.is_empty() || stop_after <= CompileStage::MirOptimize {
                        tx_to_main.send(MessageToMain::IrComplete {
                            module_path: Some(input_module_path),
                            compile_stage: CompileStage::MirOptimize,
                            errors: optimized_mir_session.errors.clone(),
                            warnings: optimized_mir_session.warnings.clone(),
                        })?;

                        if !optimized_mir_session.errors.is_empty() {
                            return Err(Error::CompileError);
                        }

                        continue;
                    }

                    self.log_start(CompileStage::Bytecode, Some(input_module_path.to_string()));
                    let bytecode_session = sodigy_bytecode::lower(optimized_mir_session);
                    self.log_end(!bytecode_session.errors.is_empty());

                    emit_irs_if_has_to(
                        &bytecode_session,
                        &emit_ir_options,
                        CompileStage::Bytecode,
                        Some(content_hash),
                        &intermediate_dir,
                    )?;

                    if !bytecode_session.errors.is_empty() || stop_after <= CompileStage::Bytecode {
                        tx_to_main.send(MessageToMain::IrComplete {
                            module_path: Some(input_module_path),
                            compile_stage: CompileStage::Bytecode,
                            errors: bytecode_session.errors.clone(),
                            warnings: bytecode_session.warnings.clone(),
                        })?;

                        if !bytecode_session.errors.is_empty() {
                            return Err(Error::CompileError);
                        }

                        continue;
                    }

                    self.log_start(CompileStage::BytecodeOptimize, Some(input_module_path.to_string()));
                    let optimized_bytecode_session = sodigy_optimize::optimize_bytecode(bytecode_session, optimize_level);
                    self.log_end(!optimized_bytecode_session.errors.is_empty());

                    emit_irs_if_has_to(
                        &optimized_bytecode_session,
                        &emit_ir_options,
                        CompileStage::BytecodeOptimize,
                        Some(content_hash),
                        &intermediate_dir,
                    )?;

                    // bytecode optimizer doesn't emit any warning/error, and this must be the last stage!
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: Some(input_module_path),
                        compile_stage: CompileStage::BytecodeOptimize,
                        errors: optimized_bytecode_session.errors.clone(),
                        warnings: optimized_bytecode_session.warnings.clone(),
                    })?;
                },
                Command::InterHir {
                    modules,
                    intermediate_dir,
                    emit_ir_options,
                } => {
                    let mut inter_hir_session = sodigy_inter_hir::Session::new(&intermediate_dir);

                    for (path, span) in modules.iter() {
                        let file = File::from_module_path(
                            0,  // project_id
                            &path.to_string(),
                            &intermediate_dir,
                        )?.ok_or(Error::MiscError)?;
                        let content_hash = file.get_content_hash(&intermediate_dir)?;
                        let hir_session_bytes = get_cached_ir(
                            &intermediate_dir,
                            CompileStage::Hir,
                            Some(content_hash),
                        )?.ok_or(Error::IrCacheNotFound(CompileStage::Hir))?;
                        let mut hir_session = sodigy_hir::Session::decode(&hir_session_bytes)?;
                        hir_session.intermediate_dir = intermediate_dir.clone();
                        inter_hir_session.ingest(*span, hir_session);
                    }

                    self.log_start(CompileStage::InterHir, None);

                    if let Ok(()) = inter_hir_session.resolve_alias() {
                        // `resolve_associated_items` will create new poly-impls
                        if let Ok(()) = inter_hir_session.resolve_associated_items() {
                            let _ = inter_hir_session.resolve_poly();
                        }
                    }

                    let has_error = !inter_hir_session.errors.is_empty();
                    self.log_end(has_error);
                    emit_irs_if_has_to(
                        &inter_hir_session,
                        &emit_ir_options,
                        CompileStage::InterHir,
                        None,
                        &intermediate_dir,
                    )?;

                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: None,
                        compile_stage: CompileStage::InterHir,
                        errors: inter_hir_session.errors,
                        warnings: inter_hir_session.warnings,
                    })?;

                    if has_error {
                        return Err(Error::CompileError);
                    }
                },
                Command::InterMir {
                    modules,
                    intermediate_dir,
                    emit_ir_options,
                } => {
                    let mut merged_mir_session: Option<mir::Session> = None;

                    for path in modules.keys() {
                        let file = File::from_module_path(
                            0,  // project_id
                            &path.to_string(),
                            &intermediate_dir,
                        )?.ok_or(Error::MiscError)?;
                        let content_hash = file.get_content_hash(&intermediate_dir)?;
                        let mir_session_bytes = get_cached_ir(
                            &intermediate_dir,
                            CompileStage::Mir,
                            Some(content_hash),
                        )?.ok_or(Error::IrCacheNotFound(CompileStage::Mir))?;
                        let mut mir_session = sodigy_mir::Session::decode(&mir_session_bytes)?;
                        mir_session.intermediate_dir = intermediate_dir.clone();

                        match &mut merged_mir_session {
                            Some(s) => {
                                s.merge(mir_session);
                            },
                            None => {
                                merged_mir_session = Some(mir_session);
                            },
                        }
                    }

                    let mir_session = merged_mir_session.unwrap();

                    // `inter_mir_session` has type information of every items in the project.
                    // It's relatively cheap to load/store, so post-mir and later stages will
                    // use this session to get type information.
                    //
                    // `mir_session` has definition of every items, after poly-solving and
                    // monomorphization. It's very heavy, and we're not gonna store this.
                    self.log_start(CompileStage::InterMir, None);
                    let (inter_mir_session, mut mir_session) = sodigy_inter_mir::solve_type(mir_session);
                    self.log_end(!inter_mir_session.errors.is_empty() || !mir_session.errors.is_empty());

                    init_span_string_map_if_necessary(
                        &mut mir_session,
                        &emit_ir_options,
                        &intermediate_dir,
                        /* read_from_file: */ false,
                        /* write_to_file: */ true,
                    )?;

                    // InterMir may have modified MIRs, so we have to update all the cached MIRs.
                    // NOTE: It drains the items in `mir_session`, so we cannot use the session anymore.
                    // TODO: This is (potentially) one of the biggest bottleneck in the compiler.
                    let items = mir_session.get_item_map();

                    for path in modules.keys() {
                        let file = File::from_module_path(
                            0,  // project_id
                            &path.to_string(),
                            &intermediate_dir,
                        )?.ok_or(Error::MiscError)?;
                        let content_hash = file.get_content_hash(&intermediate_dir)?;
                        let mir_session_bytes = get_cached_ir(
                            &intermediate_dir,
                            CompileStage::Mir,
                            Some(content_hash),
                        )?.ok_or(Error::IrCacheNotFound(CompileStage::Mir))?;
                        let mut mir_session = sodigy_mir::Session::decode(&mir_session_bytes)?;
                        mir_session.intermediate_dir = intermediate_dir.clone();
                        mir_session.update_items(&items);
                        emit_irs_if_has_to(
                            &mir_session,
                            &[
                                EmitIrOption {
                                    stage: CompileStage::Mir,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: false,
                                },
                            ],
                            CompileStage::Mir,
                            Some(content_hash),
                            &intermediate_dir,
                        )?;
                    }

                    emit_irs_if_has_to(
                        &inter_mir_session,
                        &emit_ir_options,
                        CompileStage::InterMir,
                        None,
                        &intermediate_dir,
                    )?;

                    let has_error = !inter_mir_session.errors.is_empty();
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: None,
                        compile_stage: CompileStage::InterMir,
                        errors: inter_mir_session.errors,
                        warnings: inter_mir_session.warnings,
                    })?;

                    if has_error {
                        return Err(Error::CompileError);
                    }
                },
                Command::CodeGen {
                    modules,
                    intermediate_dir,
                    backend,
                    output_path,
                } => {
                    let mut merged_bytecode_session: Option<sodigy_bytecode::Session> = None;

                    for path in modules.keys() {
                        let file = File::from_module_path(
                            0,  // project_id
                            &path.to_string(),
                            &intermediate_dir,
                        )?.ok_or(Error::MiscError)?;
                        let content_hash = file.get_content_hash(&intermediate_dir)?;
                        let bytecode_session_bytes = get_cached_ir(
                            &intermediate_dir,
                            CompileStage::BytecodeOptimize,
                            Some(content_hash),
                        )?.ok_or(Error::IrCacheNotFound(CompileStage::BytecodeOptimize))?;
                        let mut bytecode_session = sodigy_bytecode::Session::decode(&bytecode_session_bytes)?;
                        bytecode_session.intermediate_dir = intermediate_dir.clone();

                        match &mut merged_bytecode_session {
                            Some(s) => {
                                s.merge(bytecode_session);
                            },
                            None => {
                                merged_bytecode_session = Some(bytecode_session);
                            },
                        }
                    }

                    let bytecode_session = merged_bytecode_session.unwrap();
                    self.log_start(CompileStage::CodeGen, None);
                    let (code, errors, warnings) = sodigy_code_gen::lower(bytecode_session, backend);
                    self.log_end(!errors.is_empty());

                    match output_path {
                        StoreIrAt::File(f) => {
                            write_bytes(&f, &code.encode(), WriteMode::CreateOrTruncate)?;
                        },
                        StoreIrAt::IntermediateDir => {
                            emit_irs_if_has_to(
                                &code,
                                &[EmitIrOption {
                                    stage: CompileStage::CodeGen,
                                    store: StoreIrAt::IntermediateDir,
                                    human_readable: false,
                                }],
                                CompileStage::CodeGen,
                                None,
                                &intermediate_dir,
                            )?;
                        },
                    }

                    let has_error = !errors.is_empty();
                    tx_to_main.send(MessageToMain::IrComplete {
                        module_path: None,
                        compile_stage: CompileStage::CodeGen,
                        errors,
                        warnings,
                    })?;

                    if has_error {
                        return Err(Error::CompileError);
                    }
                },
            }
        }

        Ok(())
    }
}

fn init_span_string_map_if_necessary(
    session: &mut mir::Session,
    emit_ir_options: &[EmitIrOption],
    intermediate_dir: &str,
    read_from_file: bool,
    write_to_file: bool,
) -> Result<(), Error> {
    for option in emit_ir_options.iter() {
        match option {
            EmitIrOption {
                stage: CompileStage::Mir | CompileStage::InterMir | CompileStage::PostMir,
                human_readable: true,
                ..
            } => {
                let path = join3(
                    intermediate_dir,
                    "irs",
                    "span_string_map",
                )?;

                if read_from_file {
                    let bytes = read_bytes(&path)?;
                    session.span_string_map = Some(HashMap::<_, _>::decode(&bytes)?);
                }

                else {
                    session.init_span_string_map();
                }

                if write_to_file {
                    let Some(span_string_map) = &session.span_string_map else { unreachable!() };
                    let bytes = span_string_map.encode();
                    write_bytes(&path, &bytes, WriteMode::CreateOrTruncate)?;
                }

                break;
            },
            _ => {},
        }
    }

    Ok(())
}
