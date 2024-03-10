use crate::result::CompilerOutput;
use sodigy_error::UniversalError;
use sodigy_high_ir::HirSession;
use std::collections::HashMap;
use std::sync::Mutex;

type Path = String;

static mut GLOBAL_HIR_CACHE_PTR: *mut GlobalHirCache = std::ptr::null_mut();
static mut IS_GLOBAL_HIR_CACHE_INIT: bool = false;
static mut GLOBAL_HIR_CACHE_LOCK: Mutex<()> = Mutex::new(());

/// HirSessions are constructed in parallel.
/// The parallel workers this cache to organize their jobs.
pub struct GlobalHirCache {
    // keys are the names of the modules, not the paths.
    // It's `foo`, not `./foo.sdg`. `import foo;` points to the same file regardless of the path of the file it's currently compiling.
    hir_sessions: HashMap<String, (Option<HirSession>, CompilerOutput)>,
    hir_sessions_to_read: HashMap<String, Path>,
    has_error: bool,
}

impl GlobalHirCache {
    pub fn new() -> Self {
        GlobalHirCache {
            hir_sessions: HashMap::new(),
            hir_sessions_to_read: HashMap::new(),
            has_error: false,
        }
    }

    pub fn pop_job_queue(&mut self) -> Option<(String, Path)> {
        let lock = unsafe { GLOBAL_HIR_CACHE_LOCK.lock().unwrap() };
        let mut iterator = self.hir_sessions_to_read.iter();
        let result = iterator.next().map(|(n, p)| (n.clone(), p.clone()));

        if let Some((name, _)) = &result {
            self.hir_sessions_to_read.remove(name);
        }

        drop(lock);

        result
    }

    // TODO: it has to reject when `path == base_path`
    pub fn push_job_queue(&mut self, name: String, path: Path) {
        let lock = unsafe { GLOBAL_HIR_CACHE_LOCK.lock().unwrap() };

        if !self.hir_sessions.contains_key(&name) {
            self.hir_sessions_to_read.insert(name, path);
        }

        drop(lock);
    }

    pub fn push_result(&mut self, name: String, result: (Option<HirSession>, CompilerOutput)) {
        let lock = unsafe { GLOBAL_HIR_CACHE_LOCK.lock().unwrap() };

        if result.0.is_none() {
            self.has_error = true;
        }

        self.hir_sessions.insert(name, result);

        drop(lock);
    }

    pub fn has_error(&self) -> bool {
        let lock = unsafe { GLOBAL_HIR_CACHE_LOCK.lock().unwrap() };
        let result = self.has_error;
        drop(lock);

        result
    }

    pub fn collect_all_errors_and_warnings(&self) -> Vec<UniversalError> {
        todo!()
    }
}

pub unsafe fn init_global_hir_cache() -> &'static mut GlobalHirCache {
    if IS_GLOBAL_HIR_CACHE_INIT {
        return get_global_hir_cache();
    }

    let lock = GLOBAL_HIR_CACHE_LOCK.lock().unwrap();

    // another thread might init the cache while the lock is being acquired
    if IS_GLOBAL_HIR_CACHE_INIT {
        return get_global_hir_cache();
    }

    let mut result = Box::new(GlobalHirCache::new());
    GLOBAL_HIR_CACHE_PTR = result.as_mut() as *mut GlobalHirCache;
    IS_GLOBAL_HIR_CACHE_INIT = true;
    drop(lock);

    std::mem::forget(result);

    get_global_hir_cache()
}

pub unsafe fn get_global_hir_cache() -> &'static mut GlobalHirCache {
    if !IS_GLOBAL_HIR_CACHE_INIT {
        return init_global_hir_cache();
    }

    GLOBAL_HIR_CACHE_PTR.as_mut().unwrap()
}
