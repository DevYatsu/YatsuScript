//! String operations: upper, lower, trim, starts_with, ends_with,
//! contains, replace, split, repeat, slice, index_of, to_number,
//! is_empty, chars.
//!
//! Each function takes the receiver string as its first argument
//! (the value that was piped into it).

use crate::context::NativeCtx;
use crate::heap::ManagedObject;
use crate::natives::NativeRegistry;
use crate::natives::alloc_string_native;
use std::borrow::Cow;
use ys_core::compiler::Value;
use ys_core::error::JitError;

/// Extract the string value from `args[0]` using `value_as_string`.
fn get_string(ctx: &NativeCtx, args: &[Value], name: &str) -> Result<String, JitError> {
    let val = args.first().copied().unwrap_or(Value::nil());
    ctx.value_as_string(val)
        .map(Cow::into_owned)
        .ok_or_else(|| {
            JitError::runtime(
                format!("{}: expected a string as first argument", name),
                (0, 0),
            )
        })
}

pub(crate) fn register(reg: &mut NativeRegistry) {
    reg.insert("upper", |ctx, args| {
        let s = get_string(ctx, args, "upper")?;
        Ok(alloc_string_native(ctx, s.to_uppercase()))
    });

    reg.insert("lower", |ctx, args| {
        let s = get_string(ctx, args, "lower")?;
        Ok(alloc_string_native(ctx, s.to_lowercase()))
    });

    reg.insert("trim", |ctx, args| {
        let s = get_string(ctx, args, "trim")?;
        Ok(alloc_string_native(ctx, s.trim()))
    });

    reg.insert("starts_with", |ctx, args| {
        let s = get_string(ctx, args, "starts_with")?;
        let pattern = args
            .get(1)
            .and_then(|v| ctx.value_as_string(*v))
            .unwrap_or_default();
        Ok(Value::bool(s.starts_with(pattern.as_ref())))
    });

    reg.insert("ends_with", |ctx, args| {
        let s = get_string(ctx, args, "ends_with")?;
        let pattern = args
            .get(1)
            .and_then(|v| ctx.value_as_string(*v))
            .unwrap_or_default();
        Ok(Value::bool(s.ends_with(pattern.as_ref())))
    });

    reg.insert("contains", |ctx, args| {
        let s = get_string(ctx, args, "contains")?;
        let pattern = args
            .get(1)
            .and_then(|v| ctx.value_as_string(*v))
            .unwrap_or_default();
        Ok(Value::bool(s.contains(pattern.as_ref())))
    });

    reg.insert("replace", |ctx, args| {
        let s = get_string(ctx, args, "replace")?;
        let from = args
            .get(1)
            .and_then(|v| ctx.value_as_string(*v))
            .map(Cow::into_owned)
            .unwrap_or_default();
        let to = args
            .get(2)
            .and_then(|v| ctx.value_as_string(*v))
            .map(Cow::into_owned)
            .unwrap_or_default();
        Ok(alloc_string_native(ctx, s.replace(&from, &to)))
    });

    reg.insert("split", |ctx, args| {
        let s = get_string(ctx, args, "split")?;
        let delim = args
            .get(1)
            .and_then(|v| ctx.value_as_string(*v))
            .map(Cow::into_owned)
            .unwrap_or_default();
        let parts: Vec<Value> = if delim.is_empty() {
            s.chars()
                .map(|c| alloc_string_native(ctx, c.to_string()))
                .collect()
        } else {
            s.split(&delim)
                .map(|part| alloc_string_native(ctx, part))
                .collect()
        };
        Ok(ctx.alloc(ManagedObject::List(parts)))
    });

    reg.insert("repeat", |ctx, args| {
        let s = get_string(ctx, args, "repeat")?;
        let n = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
        Ok(alloc_string_native(ctx, s.repeat(n)))
    });

    reg.insert("slice", |ctx, args| {
        let s = get_string(ctx, args, "slice")?;
        let start = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as usize;
        let end = args
            .get(2)
            .and_then(|v| v.as_number())
            .map(|n| n as usize)
            .unwrap_or(s.len());
        let (start, end) = (start.min(s.len()), end.min(s.len()));
        Ok(alloc_string_native(ctx, &s[start..end]))
    });

    reg.insert("index_of", |ctx, args| {
        let s = get_string(ctx, args, "index_of")?;
        let pattern = args
            .get(1)
            .and_then(|v| ctx.value_as_string(*v))
            .unwrap_or_default();
        Ok(Value::number(
            s.find(pattern.as_ref()).map(|i| i as f64).unwrap_or(-1.0),
        ))
    });

    reg.insert("to_number", |ctx, args| {
        let s = get_string(ctx, args, "to_number")?;
        Ok(Value::number(s.parse::<f64>().unwrap_or(0.0)))
    });

    reg.insert("is_empty", |ctx, args| {
        let s = get_string(ctx, args, "is_empty")?;
        Ok(Value::bool(s.is_empty()))
    });

    reg.insert("chars", |ctx, args| {
        let s = get_string(ctx, args, "chars")?;
        let chars: Vec<Value> = s
            .chars()
            .map(|c| alloc_string_native(ctx, c.to_string()))
            .collect();
        Ok(ctx.alloc(ManagedObject::List(chars)))
    });
}
