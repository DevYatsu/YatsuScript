//! Collection built-ins: `len`.

use crate::heap::ManagedObject;
use crate::natives::NativeRegistry;
use ys_core::compiler::Value;
use ys_core::error::JitError;

pub(crate) fn register(reg: &mut NativeRegistry) {
    reg.insert("len", |ctx, args| {
        let [val] = args else {
            return Err(JitError::runtime("len() expects 1 argument", (0, 0)));
        };
        let val = *val;

        if let Some(oid) = val.as_obj_id() {
            let heap = ctx.heap_objects();
            if let Some(Some(obj)) = heap.get(oid as usize) {
                return Ok(Value::number(match &obj.obj {
                    ManagedObject::String(s) => s.len() as f64,
                    ManagedObject::List(l) => l.len() as f64,
                    ManagedObject::Object(d) => d.map.len() as f64,
                    ManagedObject::Range { start, end, step } => {
                        if *step == 0.0 {
                            0.0
                        } else {
                            ((end - start) / step).ceil().max(0.0)
                        }
                    }
                    ManagedObject::Timestamp(_)
                    | ManagedObject::BoundMethod { .. }
                    | ManagedObject::Closure(_)
                    | ManagedObject::Promise(_) => 0.0,
                }));
            }
        } else if let Some(s) = ctx.value_as_string(val) {
            return Ok(Value::number(s.len() as f64));
        }

        Err(JitError::runtime("len() expects string or list", (0, 0)))
    });
}
