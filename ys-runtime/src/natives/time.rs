//! Time built-ins: `time`, `timestamp`, `sleep`.

use crate::context::NativeFn;
use crate::heap::ManagedObject;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use ys_core::compiler::Value;
use ys_core::error::JitError;

pub fn register(fns: &mut FxHashMap<String, NativeFn>) {
    fns.insert("time".into(), Arc::new(|_, _, _| {
        Box::pin(async move {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            Ok(Value::number(now))
        })
    }));

    fns.insert("timestamp".into(), Arc::new(|ctx, _, _| {
        Box::pin(async move {
            let temp = AtomicU64::new(0);
            ctx.alloc(ManagedObject::Timestamp(std::time::Instant::now()), &temp);
            Ok(Value::from_bits(temp.load(Ordering::Relaxed)))
        })
    }));

    fns.insert("sleep".into(), Arc::new(|_, args, loc| {
        Box::pin(async move {
            let [val] = args.as_slice() else {
                return Err(JitError::runtime(
                    "sleep() expects 1 argument",
                    loc.line as usize,
                    loc.col as usize,
                ));
            };
            let ms = val.as_number().ok_or_else(|| JitError::runtime(
                "sleep() expects numeric milliseconds",
                loc.line as usize,
                loc.col as usize,
            ))?;
            tokio::time::sleep(tokio::time::Duration::from_millis(ms as u64)).await;
            Ok(Value::from_bits(0))
        })
    }));
}
