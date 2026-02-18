use crate::compiler::{Loc, Program, Value};
use crate::error::JitError;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub mod interpreter;

pub trait Backend {
    fn run(&self, program: Program) -> Pin<Box<dyn Future<Output = Result<(), JitError>> + Send>>;
}

/// Represents the age of an object in the generational garbage collector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Generation {
    /// Newly allocated objects start here.
    Nursery,
    /// Objects that survive at least one GC cycle are promoted to the Tenured generation.
    Tenured,
}

/// A heap-allocated object managed by the garbage collector.
pub enum ManagedObject {
    /// A UTF-8 string.
    String(Arc<str>),
    /// A fixed-size list of values, where each element is an atomic 64-bit word.
    List(Box<[AtomicU64]>),
}

/// Metadata and storage for an object on the heap.
pub struct HeapObject {
    /// The actual object data.
    pub obj: ManagedObject,
    /// The ID of the last GC cycle that visited this object (used for marking).
    pub last_gc_id: u32,
    /// The generation of this object (Nursery or Tenured).
    pub generation: Generation,
}

/// The execution context shared across all threads and tasks.
///
/// It contains the global state, the heap, the string pool, and metadata
/// required for synchronization and garbage collection.
pub struct Context {
    /// Global variables shared by all tasks.
    pub globals: Vec<AtomicU64>,
    /// Interned string pool.
    pub string_pool: Arc<[Arc<str>]>,
    /// The shared heap, protected by a Read-Write lock.
    pub heap: RwLock<Vec<Option<HeapObject>>>,
    /// List of indices in the heap that are currently free/available for reuse.
    pub free_list: Mutex<Vec<u32>>,
    /// List of object IDs in the Nursery generation (used for Minor GC).
    pub nursery_ids: Mutex<Vec<u32>>,
    /// Registered native functions mapped by their ID.
    pub native_fns: Vec<Option<NativeFn>>,
    /// Tracks active register sets for all running tasks (used as GC roots).
    pub active_registers: RwLock<Vec<Arc<[AtomicU64]>>>,
    /// Set of tenured objects that point to objects in the nursery.
    pub remembered_set: Mutex<rustc_hash::FxHashSet<u32>>,
    /// Monotonically increasing counter of GC cycles performed.
    pub gc_count: std::sync::atomic::AtomicU32,
    /// Number of allocations performed since the last garbage collection.
    pub alloc_since_gc: std::sync::atomic::AtomicUsize,
    /// The user-defined functions compiled into bytecode.
    pub functions: Arc<[crate::compiler::UserFunction]>,
}

impl Context {
    pub fn alloc(&self, obj: ManagedObject, dst: &AtomicU64) -> u32 {
        {
            let count = self.alloc_since_gc.fetch_add(1, Ordering::Relaxed);
            if count >= 10000 {
                if self
                    .alloc_since_gc
                    .compare_exchange(count + 1, 0, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    println!("DEBUG: Triggering GC...");
                    self.collect_garbage();
                }
            }
        }

        let mut heap = self.heap.write().unwrap();
        let id = {
            let mut free_list = self.free_list.lock().unwrap();
            let id = if let Some(id) = free_list.pop() {
                heap[id as usize] = Some(HeapObject {
                    obj,
                    last_gc_id: 0,
                    generation: Generation::Nursery,
                });
                id
            } else {
                let id = heap.len() as u32;
                heap.push(Some(HeapObject {
                    obj,
                    last_gc_id: 0,
                    generation: Generation::Nursery,
                }));
                id
            };
            // Root it immediately while holding the heap lock
            dst.store(Value::object(id).to_bits(), Ordering::Relaxed);
            id
        };

        let mut nursery = self.nursery_ids.lock().unwrap();
        nursery.push(id);
        id
    }

    pub fn collect_garbage(&self) {
        let gc_id = self.gc_count.fetch_add(1, Ordering::Relaxed) + 1;

        if gc_id % 5 == 0 {
            self.major_gc(gc_id);
        } else {
            self.minor_gc(gc_id);
        }
    }

    pub fn major_gc(&self, gc_id: u32) {
        let mut heap = self.heap.write().unwrap();
        let mut worklist = Vec::new();

        self.trace_roots(&mut worklist);

        while let Some(id) = worklist.pop() {
            if let Some(Some(obj)) = heap.get_mut(id as usize) {
                if obj.last_gc_id != gc_id {
                    obj.last_gc_id = gc_id;
                    self.trace_object_ids(obj, &mut worklist);
                }
            }
        }

        let mut free_list = self.free_list.lock().unwrap();
        let mut remembered_set = self.remembered_set.lock().unwrap();
        let mut nursery_ids = self.nursery_ids.lock().unwrap();

        remembered_set.clear();
        nursery_ids.clear();

        for i in 0..heap.len() {
            if let Some(ref mut obj) = heap[i] {
                if obj.last_gc_id != gc_id {
                    heap[i] = None;
                    free_list.push(i as u32);
                } else {
                    obj.generation = Generation::Tenured;
                }
            }
        }
    }

    pub fn minor_gc(&self, gc_id: u32) {
        let mut heap = self.heap.write().unwrap();
        let mut worklist = Vec::new();

        self.trace_roots(&mut worklist);
        {
            let remembered = self.remembered_set.lock().unwrap();
            for &id in remembered.iter() {
                worklist.push(id);
            }
        }

        while let Some(id) = worklist.pop() {
            if let Some(Some(obj)) = heap.get_mut(id as usize) {
                if obj.last_gc_id != gc_id {
                    obj.last_gc_id = gc_id;
                    self.trace_object_ids(obj, &mut worklist);
                }
            }
        }

        let mut free_list = self.free_list.lock().unwrap();
        let mut nursery_ids = self.nursery_ids.lock().unwrap();
        let mut promoted_ids = Vec::new();

        for id in nursery_ids.drain(..) {
            if let Some(Some(obj)) = heap.get_mut(id as usize) {
                if obj.last_gc_id != gc_id {
                    heap[id as usize] = None;
                    free_list.push(id);
                } else {
                    obj.generation = Generation::Tenured;
                    promoted_ids.push(id);
                }
            }
        }

        let mut remembered_set = self.remembered_set.lock().unwrap();
        let mut new_remembered = rustc_hash::FxHashSet::default();

        for &id in remembered_set.iter() {
            if let Some(Some(obj)) = heap.get(id as usize) {
                if obj.generation == Generation::Tenured && self.check_points_to_nursery(obj, &heap)
                {
                    new_remembered.insert(id);
                }
            }
        }

        for id in promoted_ids {
            if let Some(Some(obj)) = heap.get(id as usize) {
                if self.check_points_to_nursery(obj, &heap) {
                    new_remembered.insert(id);
                }
            }
        }

        *remembered_set = new_remembered;
    }

    fn trace_roots(&self, worklist: &mut Vec<u32>) {
        // Trace string pool literals (the first N objects in the heap)
        for i in 0..self.string_pool.len() {
            worklist.push(i as u32);
        }

        for global in &self.globals {
            let val = Value::from_bits(global.load(Ordering::Relaxed));
            if let Some(id) = val.as_obj_id() {
                worklist.push(id);
            }
        }

        let active_regs = self.active_registers.read().unwrap();
        for regs in active_regs.iter() {
            for atomic_val in regs.iter() {
                let val = Value::from_bits(atomic_val.load(Ordering::Relaxed));
                if let Some(id) = val.as_obj_id() {
                    worklist.push(id);
                }
            }
        }
    }

    fn trace_object_ids(&self, obj: &HeapObject, worklist: &mut Vec<u32>) {
        if let ManagedObject::List(elements) = &obj.obj {
            for atomic_v in elements.iter() {
                let v = Value::from_bits(atomic_v.load(Ordering::Relaxed));
                if let Some(child_id) = v.as_obj_id() {
                    worklist.push(child_id);
                }
            }
        }
    }

    pub fn check_points_to_nursery(&self, obj: &HeapObject, heap: &[Option<HeapObject>]) -> bool {
        if let ManagedObject::List(elements) = &obj.obj {
            for atomic_v in elements.iter() {
                let v = Value::from_bits(atomic_v.load(Ordering::Relaxed));
                if let Some(child_id) = v.as_obj_id()
                    && let Some(Some(child)) = heap.get(child_id as usize)
                    && child.generation == Generation::Nursery
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_string_value(&self, val: Value) -> Option<String> {
        let bits = val.to_bits();
        let tag = (bits & crate::compiler::TAG_MASK) >> 48;
        if (3..=9).contains(&tag) {
            let len = (tag - 3) as usize;
            let mut bytes = Vec::with_capacity(len);
            for i in 0..len {
                bytes.push(((bits >> (i * 8)) & 0xFF) as u8);
            }
            return Some(String::from_utf8_lossy(&bytes).to_string());
        }
        if let Some(oid) = val.as_obj_id() {
            let heap = self.heap.read().unwrap();
            if let Some(Some(obj)) = heap.get(oid as usize) {
                if let ManagedObject::String(s) = &obj.obj {
                    return Some(s.to_string());
                }
            }
        }
        None
    }

    pub fn values_equal(&self, v1: Value, v2: Value) -> bool {
        let b1 = v1.to_bits();
        let b2 = v2.to_bits();
        if b1 == b2 {
            return true;
        }

        if let (Some(n1), Some(n2)) = (v1.as_number(), v2.as_number()) {
            return n1 == n2;
        }

        // Try SSO comparison
        let tag1 = (b1 & crate::compiler::TAG_MASK) >> 48;
        let tag2 = (b2 & crate::compiler::TAG_MASK) >> 48;

        if (3..=9).contains(&tag1) || (3..=9).contains(&tag2) {
            let s1 = self.get_string_value(v1);
            let s2 = self.get_string_value(v2);
            return match (s1, s2) {
                (Some(s1), Some(s2)) => s1 == s2,
                _ => false,
            };
        }

        // Both could be heap strings
        if let (Some(id1), Some(id2)) = (v1.as_obj_id(), v2.as_obj_id()) {
            let heap = self.heap.read().unwrap();
            if id1 < heap.len() as u32 && id2 < heap.len() as u32 {
                if let (Some(o1), Some(o2)) = (&heap[id1 as usize], &heap[id2 as usize]) {
                    if let (ManagedObject::String(s1), ManagedObject::String(s2)) =
                        (&o1.obj, &o2.obj)
                    {
                        return s1 == s2;
                    }
                }
            }
        }

        false
    }
}

pub type NativeFn = Arc<
    dyn Fn(
            Arc<Context>,
            Vec<Value>,
            Loc,
        ) -> Pin<Box<dyn Future<Output = Result<Value, JitError>> + Send>>
        + Send
        + Sync,
>;
