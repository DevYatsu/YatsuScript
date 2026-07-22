//! Promise states and resolution helpers for the async/await runtime.
//!
//! [`PromiseState`] tracks the lifecycle of a promise: pending (with an
//! optional continuation), resolved, rejected, or compound (awaiting multiple
//! sub-promises).

use crate::context::Context;
use crate::heap::ManagedObject;
use crate::vm::FrameState;
use ys_core::compiler::Value;

/// The state of a single promise object.
pub enum PromiseState {
    /// Still pending — optionally with a continuation to resume when
    /// the promise is resolved.
    Pending {
        continuation: Option<Box<FrameState>>,
    },
    /// Successfully resolved to a value.
    Resolved(Value),
    /// Rejected with a failure name (string-pool index).
    Rejected(u32),
    /// Tracks sub-promises that must all resolve before this promise resolves.
    /// The event loop polls sub-promises each tick.
    Compound {
        /// Object IDs of all sub-promises (unresolved = Some(oid), resolved = None).
        sub_promises: Vec<Option<u32>>,
        /// Resolved values collected in order (placeholder for unresolved).
        results: Vec<Value>,
        /// Continuation to resume when all sub-promises resolve.
        continuation: Option<Box<FrameState>>,
    },
}

/// Try to extract the resolved value from a Promise.  Returns:
/// - `Ok(val)` if resolved
/// - `Err(Some(name_id))` if rejected  
/// - `Err(None)` if still pending (or compound not yet satisfied)
pub fn resolve_promise(ctx: &Context, oid: u32) -> Result<Value, Option<u32>> {
    let objects = ctx.heap.objects.get();
    if let Some(Some(obj)) = objects.get(oid as usize) {
        match &obj.obj {
            ManagedObject::Promise(PromiseState::Resolved(v)) => Ok(*v),
            ManagedObject::Promise(PromiseState::Rejected(name_id)) => Err(Some(*name_id)),
            ManagedObject::Promise(PromiseState::Pending { .. })
            | ManagedObject::Promise(PromiseState::Compound { .. }) => Err(None),
            _ => Err(None),
        }
    } else {
        Err(None)
    }
}

/// If `v` is a resolved Promise, return its inner value; otherwise return `v` as-is.
#[allow(clippy::result_unit_err)]
pub fn resolve_promise_value(ctx: &Context, v: Value) -> Result<Value, ()> {
    if let Some(oid) = v.as_obj_id() {
        let objects = ctx.heap.objects.get();
        if let Some(Some(obj)) = objects.get(oid as usize)
            && let ManagedObject::Promise(ps) = &obj.obj
        {
            return match ps {
                PromiseState::Resolved(val) => Ok(*val),
                _ => Err(()),
            };
        }
    }
    Ok(v)
}
