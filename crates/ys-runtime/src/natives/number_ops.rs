//! Number operations: to_string, ceil, floor, round, abs, sqrt,
//! pow, is_integer, to_int.
//!
//! Each function takes the receiver number as its first argument
//! (the value that was piped into it).

use crate::natives::NativeRegistry;
use crate::natives::alloc_string_native;
use crate::value_fmt::stringify_value;
use ys_core::compiler::Value;
use ys_core::error::JitError;

/// Extract the number value from `args[0]`.
fn get_number(args: &[Value], name: &str) -> Result<f64, JitError> {
    args.first()
        .copied()
        .unwrap_or(Value::nil())
        .as_number()
        .ok_or_else(|| {
            JitError::runtime(
                format!("{}: expected a number as first argument", name),
                (0, 0),
            )
        })
}

pub(crate) fn register(reg: &mut NativeRegistry) {
    reg.insert("to_string", |ctx, args| {
        let val = args.first().copied().unwrap_or(Value::nil());
        let s = stringify_value(ctx.as_inner(), val);
        Ok(alloc_string_native(ctx, s))
    });

    reg.insert("ceil", |_ctx, args| {
        let n = get_number(args, "ceil")?;
        Ok(Value::number(n.ceil()))
    });

    reg.insert("floor", |_ctx, args| {
        let n = get_number(args, "floor")?;
        Ok(Value::number(n.floor()))
    });

    reg.insert("round", |_ctx, args| {
        let n = get_number(args, "round")?;
        Ok(Value::number(n.round()))
    });

    reg.insert("abs", |_ctx, args| {
        let n = get_number(args, "abs")?;
        Ok(Value::number(n.abs()))
    });

    reg.insert("sqrt", |_ctx, args| {
        let n = get_number(args, "sqrt")?;
        Ok(Value::number(n.sqrt()))
    });

    reg.insert("pow", |_ctx, args| {
        let n = get_number(args, "pow")?;
        let exp = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0);
        Ok(Value::number(n.powf(exp)))
    });

    reg.insert("is_integer", |_ctx, args| {
        let n = get_number(args, "is_integer")?;
        Ok(Value::bool(n.fract() == 0.0))
    });

    reg.insert("to_int", |_ctx, args| {
        let n = get_number(args, "to_int")?;
        Ok(Value::number(n.trunc() as i64 as f64))
    });
}
