//! Heap allocation, object types, and the generational garbage collector.
//!
//! The heap stores all complex YatsuScript values that do not fit in the
//! 64-bit NaN-boxed [`Value`] inline (strings > 6 bytes, lists, objects,
//! ranges, timestamps, and bound methods).
//!
//! # Generational GC
//!
//! Objects begin life in the *nursery* generation. A minor collection only
//! scans nursery objects; a major collection (every 5th GC) scans everything.
//! Tenured objects that hold references to nursery objects are tracked in the
//! *remembered set* via a write barrier.

use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use ys_core::compiler::Value;

use crate::context::Context;

// ── Object variants ───────────────────────────────────────────────────────────

/// Every kind of value that lives on the heap.
pub enum ManagedObject {
    /// A heap-allocated string (longer than 6 bytes).
    String(Arc<str>),
    /// A growable list of NaN-boxed values.
    List(RwLock<Vec<AtomicU64>>),
    /// A hash map from interned name IDs to NaN-boxed values.
    Object(RwLock<rustc_hash::FxHashMap<u32, AtomicU64>>),
    /// A point-in-time snapshot (`Instant::now()`).
    Timestamp(std::time::Instant),
    /// An inclusive range with an optional step.
    Range { start: f64, end: f64, step: f64 },
    /// A method reference bound to a receiver (e.g. `list.pad`).
    BoundMethod { receiver: Value, name_id: u32 },
}

impl ManagedObject {
    /// Walk all object-reference children, calling `f` with each object ID.
    /// Used by the GC to trace the object graph.
    pub fn visit_children<F: FnMut(u32)>(&self, mut f: F) {
        match self {
            ManagedObject::List(elements) => {
                for v in elements.read().iter() {
                    if let Some(id) = Value::from_bits(v.load(Ordering::Relaxed)).as_obj_id() {
                        f(id);
                    }
                }
            }
            ManagedObject::Object(fields) => {
                for v in fields.read().values() {
                    if let Some(id) = Value::from_bits(v.load(Ordering::Relaxed)).as_obj_id() {
                        f(id);
                    }
                }
            }
            ManagedObject::BoundMethod { receiver, .. } => {
                if let Some(id) = receiver.as_obj_id() { f(id); }
            }
            // Leaf types — no children.
            ManagedObject::String(_) | ManagedObject::Timestamp(_)
            | ManagedObject::Range { .. } => {}
        }
    }
}

// ── Heap slot 

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Generation { Nursery, Tenured }

/// A single slot in the heap, combining the object with its GC metadata.
pub struct HeapObject {
    pub obj:        ManagedObject,
    pub last_gc_id: u32,
    pub generation: Generation,
}

// ── Heap ────

/// The managed object store with a generational GC.
pub struct Heap {
    /// All allocated objects (indexed by u32 object ID).
    pub objects:       RwLock<Vec<Option<HeapObject>>>,
    /// GC bookkeeping (free list, nursery set, remembered set).
    pub metadata:      Mutex<HeapMetadata>,
    /// Running count of GC cycles — every 5th triggers a major collection.
    pub gc_count:      AtomicU32,
    /// Allocations since the last GC (triggers GC at 100 000).
    pub alloc_since_gc: AtomicUsize,
}

/// GC bookkeeping data, kept separate so it can be lock-guarded independently.
pub struct HeapMetadata {
    pub free_list:     Vec<u32>,
    pub nursery_ids:   Vec<u32>,
    pub remembered_set: FxHashSet<u32>,
}

impl Heap {
    /// Trigger either a minor or major collection.
    pub fn collect_garbage(&self, ctx: &Context) {
        let gc_id = self.gc_count.fetch_add(1, Ordering::Relaxed) + 1;
        if gc_id.is_multiple_of(5) {
            self.major_gc(gc_id, ctx);
        } else {
            self.minor_gc(gc_id, ctx);
        }
    }

    /// Scan all live objects and free anything not reachable from roots.
    pub fn major_gc(&self, gc_id: u32, ctx: &Context) {
        let mut objects  = self.objects.write();
        let mut worklist = Vec::new();
        self.trace_roots(ctx, &mut worklist);

        while let Some(id) = worklist.pop() {
            if let Some(Some(obj)) = objects.get_mut(id as usize)
                && obj.last_gc_id != gc_id
            {
                obj.last_gc_id = gc_id;
                obj.obj.visit_children(|child_id| worklist.push(child_id));
            }
        }

        let mut meta = self.metadata.lock();
        meta.remembered_set.clear();
        meta.nursery_ids.clear();

        let freed: Vec<u32> = objects
            .par_iter_mut()
            .enumerate()
            .filter_map(|(i, slot)| {
                if let Some(obj) = slot {
                    if obj.last_gc_id != gc_id {
                        *slot = None;
                        return Some(i as u32);
                    }
                    obj.generation = Generation::Tenured;
                }
                None
            })
            .collect();
        meta.free_list.extend(freed);
    }

    /// Scan only nursery objects and objects in the remembered set.
    pub fn minor_gc(&self, gc_id: u32, ctx: &Context) {
        let mut objects  = self.objects.write();
        let mut worklist = Vec::new();
        self.trace_roots(ctx, &mut worklist);
        {
            let meta = self.metadata.lock();
            worklist.extend(meta.remembered_set.iter());
        }

        while let Some(id) = worklist.pop() {
            if let Some(Some(obj)) = objects.get_mut(id as usize)
                && obj.last_gc_id != gc_id
            {
                obj.last_gc_id = gc_id;
                obj.obj.visit_children(|child_id| worklist.push(child_id));
            }
        }

        let mut meta        = self.metadata.lock();
        let mut promoted    = Vec::new();
        let nursery_ids: Vec<u32> = meta.nursery_ids.drain(..).collect();

        for id in nursery_ids {
            if let Some(Some(obj)) = objects.get_mut(id as usize) {
                if obj.last_gc_id != gc_id {
                    objects[id as usize] = None;
                    meta.free_list.push(id);
                } else {
                    obj.generation = Generation::Tenured;
                    promoted.push(id);
                }
            }
        }

        // Rebuild the remembered set from tenured objects still pointing at nursery.
        let new_from_old: Vec<u32> = meta
            .remembered_set
            .par_iter()
            .filter(|&&id| {
                objects.get(id as usize)
                    .and_then(|s| s.as_ref())
                    .map(|o| o.generation == Generation::Tenured
                             && self.points_to_nursery(o, &objects))
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        let new_from_promoted: Vec<u32> = promoted
            .into_par_iter()
            .filter(|&id| {
                objects.get(id as usize)
                    .and_then(|s| s.as_ref())
                    .map(|o| self.points_to_nursery(o, &objects))
                    .unwrap_or(false)
            })
            .collect();

        let mut new_set = FxHashSet::default();
        new_set.extend(new_from_old);
        new_set.extend(new_from_promoted);
        meta.remembered_set = new_set;
    }

    fn trace_roots(&self, ctx: &Context, worklist: &mut Vec<u32>) {
        worklist.extend(0..ctx.string_pool.len() as u32);
        for g in ctx.globals.iter() {
            if let Some(id) = Value::from_bits(g.load(Ordering::Relaxed)).as_obj_id() {
                worklist.push(id);
            }
        }
        let tasks = ctx.active_registers.lock();
        for task in tasks.iter() {
            let stack = task.lock();
            for regs in stack.iter() {
                for v in regs.iter() {
                    if let Some(id) = Value::from_bits(v.load(Ordering::Relaxed)).as_obj_id() {
                        worklist.push(id);
                    }
                }
            }
        }
    }

    pub fn alloc(&self, obj: ManagedObject, root: &AtomicU64, ctx: &Context) {
        if self.alloc_since_gc.fetch_add(1, Ordering::Relaxed) > 100_000 {
            self.collect_garbage(ctx);
        }

        let mut meta = self.metadata.lock();
        let id = match meta.free_list.pop() {
            Some(i) => i,
            None => {
                let mut objects = self.objects.write();
                let i = objects.len() as u32;
                objects.push(None);
                i
            }
        };

        meta.nursery_ids.push(id);
        drop(meta);

        let mut objects = self.objects.write();
        objects[id as usize] = Some(HeapObject {
            obj,
            last_gc_id: 0,
            generation: Generation::Nursery,
        });

        root.store(Value::object(id).to_bits(), Ordering::Release);
    }

    /// Return `true` when `obj` holds a reference to any nursery-generation object.
    pub fn points_to_nursery(&self, obj: &HeapObject, heap: &[Option<HeapObject>]) -> bool {
        let mut found = false;
        obj.obj.visit_children(|child_id| {
            if !found
                && let Some(Some(child)) = heap.get(child_id as usize)
                && child.generation == Generation::Nursery
            {
                found = true;
            }
        });
        found
    }
}
