//! Value display helpers.
//!
//! Separating formatting from dispatch keeps the VM loop free of print logic.

use crate::context::Context;
use crate::heap::ManagedObject;
use std::sync::atomic::Ordering;
use ys_core::compiler::Value;

/// Produce a human-readable string representation of `val`.
pub fn stringify_value(ctx: &Context, val: Value) -> String {
    if let Some(n) = val.as_number() { return format_number(n); }
    if let Some(b) = val.as_bool()   { return b.to_string(); }
    if let Some(s) = ctx.value_as_string(val) { return s; }

    if let Some(oid) = val.as_obj_id() {
        let heap = ctx.heap.objects.read();
        if let Some(Some(obj)) = heap.get(oid as usize) {
            return match &obj.obj {
                ManagedObject::String(s)     => s.to_string(),
                ManagedObject::List(elems)   => format_list(ctx, &elems.read()),
                ManagedObject::Object(fields) => format_object(ctx, &fields.read()),
                ManagedObject::Timestamp(t)  => format!("Timestamp({:?})", t),
                ManagedObject::Range { start, end, step } => {
                    if *step == 1.0 { format!("{}..{}", start, end) }
                    else            { format!("{}..{}.step({})", start, end, step) }
                }
                ManagedObject::BoundMethod { receiver, name_id } => {
                    // Avoid locking the heap again inside a pool_name call.
                    let method = ctx.string_pool
                        .get(*name_id as usize)
                        .map(|s| s.as_ref())
                        .unwrap_or("");
                    format!("<bound method {} of {}>", method, stringify_value(ctx, *receiver))
                }
            };
        }
        return "null".into();
    }
    "unknown".into()
}

// ── Private formatting helpers ────────────────────────────────────────────────

fn format_number(n: f64) -> String {
    // Omit the trailing ".0" for whole numbers to match scripting conventions.
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

fn format_list(ctx: &Context, elems: &[std::sync::atomic::AtomicU64]) -> String {
    let items: Vec<String> = elems
        .iter()
        .map(|a| stringify_nested(ctx, Value::from_bits(a.load(Ordering::Relaxed))))
        .collect();
    format!("[{}]", items.join(", "))
}

fn format_object(
    ctx: &Context,
    fields: &rustc_hash::FxHashMap<u32, std::sync::atomic::AtomicU64>,
) -> String {
    let entries: Vec<String> = fields
        .iter()
        .map(|(&name_id, v)| {
            let name  = ctx.string_pool.get(name_id as usize).map(|s| s.as_ref()).unwrap_or("?");
            let value = Value::from_bits(v.load(Ordering::Relaxed));
            format!("{}: {}", name, stringify_nested(ctx, value))
        })
        .collect();
    format!("{{{}}}", entries.join(", "))
}

/// Like `stringify_value` but with abbreviated nested types and quoted strings.
fn stringify_nested(ctx: &Context, val: Value) -> String {
    if let Some(s) = ctx.value_as_string(val) { return format!("\"{}\"", s); }
    if let Some(oid) = val.as_obj_id() {
        let heap = ctx.heap.objects.read();
        if let Some(Some(obj)) = heap.get(oid as usize) {
            return match &obj.obj {
                ManagedObject::String(s)      => format!("\"{}\"", s),
                ManagedObject::List(_)        => "[...]".into(),
                ManagedObject::Object(_)      => "{...}".into(),
                ManagedObject::Timestamp(_)   => "Timestamp(...)".into(),
                ManagedObject::Range { .. }   => "Range(...)".into(),
                ManagedObject::BoundMethod { .. } => "BoundMethod(...)".into(),
            };
        }
        return "null".into();
    }
    stringify_value(ctx, val)
}
