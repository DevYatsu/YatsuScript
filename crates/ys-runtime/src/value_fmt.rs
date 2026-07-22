//! Value display helpers.
//!
//! Separating formatting from dispatch keeps the VM loop free of print logic.
//! All helpers write into a caller-provided buffer to avoid intermediate allocations.

use crate::context::Context;
use crate::heap::ManagedObject;
use std::fmt::Write;
use ys_core::compiler::Value;

/// Produce a human-readable string representation of `val`.
pub fn stringify_value(ctx: &Context, val: Value) -> String {
    let mut buf = String::new();
    write_value(ctx, val, &mut buf);
    buf
}

// ── Top-level value writer ────────────────────────────────────────────

/// Write the full representation of `val` into `buf`.
fn write_value(ctx: &Context, val: Value, buf: &mut String) {
    if let Some(n) = val.as_number() {
        write_number(n, buf);
    } else if let Some(b) = val.as_bool() {
        if b {
            buf.push_str("true");
        } else {
            buf.push_str("false");
        }
    } else if let Some(s) = ctx.value_as_string(val) {
        buf.push_str(&s);
    } else if let Some(oid) = val.as_obj_id() {
        let heap = ctx.heap.objects.get();
        if let Some(Some(obj)) = heap.get(oid as usize) {
            match &obj.obj {
                ManagedObject::String(s) => buf.push_str(s),
                ManagedObject::List(elems) => {
                    buf.push('[');
                    for (i, v) in elems.iter().enumerate() {
                        if i > 0 {
                            buf.push_str(", ");
                        }
                        write_nested(ctx, *v, buf);
                    }
                    buf.push(']');
                }
                ManagedObject::Object(d) => {
                    buf.push('{');
                    for (i, (&name_id, v)) in d.map.iter().enumerate() {
                        if i > 0 {
                            buf.push_str(", ");
                        }
                        let name = ctx
                            .string_pool
                            .get(name_id as usize)
                            .map(|s| s.as_ref())
                            .unwrap_or("?");
                        buf.push_str(name);
                        buf.push_str(": ");
                        write_nested(ctx, *v, buf);
                    }
                    buf.push('}');
                }
                ManagedObject::Timestamp(t) => {
                    let _ = write!(buf, "Timestamp({:?})", t);
                }
                ManagedObject::Range { start, end, step } => {
                    write_number(*start, buf);
                    buf.push_str("..");
                    write_number(*end, buf);
                    if *step != 1.0 {
                        buf.push_str(".step(");
                        write_number(*step, buf);
                        buf.push(')');
                    }
                }
                ManagedObject::BoundMethod { receiver, name_id } => {
                    let method = ctx
                        .string_pool
                        .get(*name_id as usize)
                        .map(|s| s.as_ref())
                        .unwrap_or("");
                    buf.push_str("<bound method ");
                    buf.push_str(method);
                    buf.push_str(" of ");
                    write_value(ctx, *receiver, buf);
                    buf.push('>');
                }
                ManagedObject::Closure(cl) => {
                    buf.push_str("<Closure#");
                    write_number(cl.name_id as f64, buf);
                    buf.push('>');
                }
                ManagedObject::Promise(_) => buf.push_str("<Promise>"),
            }
        } else {
            buf.push_str("null");
        }
    } else if let Some(name_id) = val.as_failure_id() {
        let name = ctx
            .string_pool
            .get(name_id as usize)
            .map(|s| s.as_ref())
            .unwrap_or("?");
        buf.push_str("fail(");
        buf.push_str(name);
        buf.push(')');
    } else {
        buf.push_str("unknown");
    }
}

// ── Nested value writer (abbreviated types, quoted strings) ───────────

/// Like `write_value` but abbreviates compound types into stub descriptions
/// and wraps strings in double-quotes (for use inside lists / objects).
fn write_nested(ctx: &Context, val: Value, buf: &mut String) {
    if let Some(s) = ctx.value_as_string(val) {
        buf.push('"');
        buf.push_str(&s);
        buf.push('"');
    } else if let Some(oid) = val.as_obj_id() {
        let heap = ctx.heap.objects.get();
        if let Some(Some(obj)) = heap.get(oid as usize) {
            match &obj.obj {
                ManagedObject::String(s) => {
                    buf.push('"');
                    buf.push_str(s);
                    buf.push('"');
                }
                ManagedObject::List(_) => buf.push_str("[...]"),
                ManagedObject::Object(_) => buf.push_str("{...}"),
                ManagedObject::Timestamp(_) => buf.push_str("Timestamp(...)"),
                ManagedObject::Range { .. } => buf.push_str("Range(...)"),
                ManagedObject::BoundMethod { .. } => buf.push_str("BoundMethod(...)"),
                ManagedObject::Closure(_) => buf.push_str("Closure(...)"),
                ManagedObject::Promise(_) => buf.push_str("Promise(...)"),
            }
        } else {
            buf.push_str("null");
        }
    } else {
        write_value(ctx, val, buf);
    }
}

// ── Number formatter ──────────────────────────────────────────────────

/// Write a `f64` number into `buf` without allocating an intermediate string.
/// Omits the trailing ".0" for whole numbers (matching scripting conventions).
fn write_number(n: f64, buf: &mut String) {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(buf, "{}", n as i64);
    } else {
        let _ = write!(buf, "{}", n);
    }
}
