use crate::heap::{Heap, ManagedObject};
use std::sync::Arc;
use ys_core::compiler::Value;
use parking_lot::Mutex;

//  Backend trait 

/// An execution backend for compiled YatsuScript programs.
///
/// This trait allows different execution strategies (e.g. interpreter vs potentially a JIT)
/// to be swapped out while using the same shared context and heap.
pub trait Backend: Send + Sync {
    /// Execute a compiled program.
    fn run(&self, program: ys_core::compiler::Program)
        -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ys_core::error::JitError>> + Send>>;
}

// ── Shared Execution State ───────────────────────────────────────────────────

/// Store for native function implementations.
pub type NativeFn = Arc<
    dyn Fn(
        Arc<Context>,
        Vec<Value>,
        ys_core::compiler::Loc,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, ys_core::error::JitError>> + Send>>
    + Send
    + Sync,
>;

/// Any object that can be called (either a user-defined function or a native builtin).
#[derive(Clone)]
pub enum Callable {
    User(ys_core::compiler::UserFunction),
    Native(NativeFn),
}

/// A register array that serves as a GC root for an active task.
pub type TaskRegisters = Arc<Mutex<Vec<Arc<[std::sync::atomic::AtomicU64]>>>>;

/// The global shared state for a YatsuScript execution environment.
///
/// Contains:
/// - The [`Heap`] (object storage)
/// - The [`string_pool`] (interned strings)
/// - The `globals` register array
/// - The map of all registered [`Callable`] entities
pub struct Context {
    pub heap:       Heap,
    pub string_pool: Arc<[Arc<str>]>,
    pub globals:    Arc<[std::sync::atomic::AtomicU64]>,
    pub callables:  rustc_hash::FxHashMap<u32, Callable>,
    
    /// Tracks all register arrays currently being used by active tasks
    /// so the GC can trace them as roots.
    pub active_registers: Arc<Mutex<Vec<TaskRegisters>>>,
}

impl Context {
    /// Allocate a new object on the heap, automatically triggering a GC if needed.
    ///
    /// The `root` argument is a reference to a register slot that will be updated
    /// if the object is moved during a collection.
    pub fn alloc(&self, obj: ManagedObject, root: &std::sync::atomic::AtomicU64) {
        self.heap.alloc(obj, root, self);
    }

    /// Check if two values are equal, potentially diving into the heap for objects.
    pub fn values_equal(&self, a: Value, b: Value) -> bool {
        if a.to_bits() == b.to_bits() { return true; }

        match (a.as_obj_id(), b.as_obj_id()) {
            (Some(aid), Some(bid)) => {
                let heap = self.heap.objects.read();
                if let (Some(Some(ao)), Some(Some(bo))) = (heap.get(aid as usize), heap.get(bid as usize)) {
                    match (&ao.obj, &bo.obj) {
                        (ManagedObject::String(asrc), ManagedObject::String(bsrc)) => asrc == bsrc,
                        _ => aid == bid,
                    }
                } else { aid == bid }
            }
            _ => false,
        }
    }

    /// Retrieve a callable by its interned string ID.
    pub fn get_callable(&self, id: u32) -> Option<&Callable> {
        self.callables.get(&id)
    }

    /// Try to read a value as a string (either SSO or from the heap).
    pub fn value_as_string(&self, v: Value) -> Option<String> {
        if let Some(s) = v.as_sso_str() { 
            return std::str::from_utf8(&s).ok().map(|s| s.to_string());
        }
        if let Some(oid) = v.as_obj_id() {
            let heap = self.heap.objects.read();
            if let Some(Some(obj)) = heap.get(oid as usize) {
                if let ManagedObject::String(s) = &obj.obj {
                    return Some(s.to_string());
                }
            }
        }
        None
    }

    /// Convert a value to its interned pool ID if possible.
    pub fn value_as_pool_id(&self, v: Value) -> Option<u32> {
        if let Some(oid) = v.as_obj_id() { return Some(oid); }
        if let Some(s) = v.as_sso_str() {
            let s_str = std::str::from_utf8(&s).ok()?;
            return self.string_pool.iter().position(|p| &**p == s_str).map(|i| i as u32);
        }
        None
    }
}
