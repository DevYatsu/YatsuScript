//! Function, closure, and async function code generation.
//!
//! These are free functions that take `&mut Codegen` so they can be defined in
//! a child module without creating a circular dependency.

use super::{Codegen, VarInfo};
use crate::ast::*;
use crate::compiler::*;
use crate::error::JitError;
use rustc_hash::FxHashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Saved state from [`begin_function`] — must be passed to [`end_function`].
struct FuncFrame {
    locals: FxHashMap<String, VarInfo>,
    next_reg: usize,
    is_in_function: bool,
    saved_instrs: Vec<Instruction>,
}

/// Save the parent context and set up registers for function compilation.
/// Must be paired with [`end_function`].
fn begin_function(cg: &mut Codegen, params: &[String]) -> FuncFrame {
    let frame = FuncFrame {
        locals: std::mem::take(&mut cg.locals),
        next_reg: cg.next_reg,
        is_in_function: cg.is_in_function,
        saved_instrs: std::mem::take(&mut cg.instructions),
    };
    // Clear per-function state — the outer scope's register space must
    // not leak into the function's own allocation.
    cg.freed_regs.clear();
    cg.var_mask = 0;
    cg.is_in_function = true;
    cg.next_reg = 0;
    for (i, p) in params.iter().enumerate() {
        cg.locals.insert(
            p.clone(),
            VarInfo {
                idx: i,
                is_global: false,
            },
        );
        cg.var_mask |= 1 << i;
        cg.next_reg = i + 1;
    }
    frame
}

/// Ensure the compiled body ends with a Return, then push the function into
/// the program's function list and restore the parent compilation context.
fn end_function(cg: &mut Codegen, name: &str, params_count: usize, frame: FuncFrame, loc: Loc) {
    if !matches!(cg.instructions.last(), Some(Instruction::Return { .. })) {
        cg.emit(Instruction::Return { value: None, loc });
    }
    let name_id = cg.intern(name);
    let func_body = std::mem::replace(&mut cg.instructions, frame.saved_instrs);
    let locals_count = cg.next_reg;

    cg.locals = frame.locals;
    cg.next_reg = frame.next_reg;
    cg.is_in_function = frame.is_in_function;

    let idx = cg.functions.len();
    cg.functions.push(UserFunction {
        name_id,
        params_count,
        locals_count,
        instructions: Arc::from(func_body),
    });
    cg.function_map.insert(name.to_string(), idx);
}

// ---------------------------------------------------------------------------
// Function declaration
// ---------------------------------------------------------------------------

pub(super) fn compile_func(
    cg: &mut Codegen,
    name: &str,
    params: &[String],
    body: &[AstNode],
    loc: Loc,
) {
    let frame = begin_function(cg, params);
    if cg.compile_block(body).is_err() {
        cg.emit(Instruction::Return { value: None, loc });
    }
    end_function(cg, name, params.len(), frame, loc);
}

// ---------------------------------------------------------------------------
// Async function
// ---------------------------------------------------------------------------

/// Compile an async function — creates a pending return promise at the
/// start so callers immediately get a Promise, even if the body suspends
/// on an internal await.  The promise is resolved when the body returns.
pub(super) fn compile_async_func(
    cg: &mut Codegen,
    name: &str,
    params: &[String],
    body: &[AstNode],
    loc: Loc,
) {
    let frame = begin_function(cg, params);
    let ret_promise_reg = cg.alloc_reg();
    cg.emit(Instruction::MakePendingPromise {
        dst: ret_promise_reg,
    });

    if cg.compile_block(body).is_err() {
        cg.emit(Instruction::Return { value: None, loc });
    }
    // Replace the final return with ResolvePromise + Return(ret_promise)
    if let Some(Instruction::Return { value: reg, .. }) = cg.instructions.pop() {
        if let Some(value_reg) = reg {
            cg.emit(Instruction::ResolvePromise {
                promise: ret_promise_reg,
                value: value_reg,
            });
        } else {
            // No return value — resolve with nil
            let nil_reg = cg.alloc_reg();
            cg.emit(Instruction::LoadLiteral {
                dst: nil_reg,
                val: Value::nil(),
                loc,
            });
            cg.emit(Instruction::ResolvePromise {
                promise: ret_promise_reg,
                value: nil_reg,
            });
        }
        cg.emit(Instruction::Return {
            value: Some(ret_promise_reg),
            loc,
        });
    } else {
        // No return instruction at all — body ran to end without returning
        let nil_reg = cg.alloc_reg();
        cg.emit(Instruction::LoadLiteral {
            dst: nil_reg,
            val: Value::nil(),
            loc,
        });
        cg.emit(Instruction::ResolvePromise {
            promise: ret_promise_reg,
            value: nil_reg,
        });
        cg.emit(Instruction::Return {
            value: Some(ret_promise_reg),
            loc,
        });
    }

    end_function(cg, name, params.len(), frame, loc);
}

// ---------------------------------------------------------------------------
// Closure
// ---------------------------------------------------------------------------

pub(super) fn compile_closure(
    cg: &mut Codegen,
    params: &[String],
    body: &AstNode,
    loc: Loc,
) -> Result<usize, JitError> {
    let mut func = Codegen::new();
    func.closure_counter = cg.closure_counter;
    func.is_in_function = true;
    for (i, p) in params.iter().enumerate() {
        func.locals.insert(
            p.clone(),
            VarInfo {
                idx: i,
                is_global: false,
            },
        );
        func.next_reg = i + 1;
    }
    let captures: Vec<String> = Vec::new();

    let result_reg = func.compile_node(body).unwrap_or(0);

    for mut nested in std::mem::take(&mut func.functions) {
        let new_name = format!("__closure_{}", cg.closure_counter);
        cg.closure_counter += 1;
        nested.name_id = cg.intern(&new_name);
        cg.functions.push(nested);
    }
    cg.closure_counter = std::cmp::max(cg.closure_counter, func.closure_counter);

    for instr in &mut func.instructions {
        match instr {
            Instruction::MakeClosure { name_id, .. }
            | Instruction::ObjectGet { name_id, .. }
            | Instruction::ObjectSet { name_id, .. } => {
                if let Some(name) = func.string_pool.get(*name_id as usize) {
                    *name_id = cg.intern(name);
                }
            }
            Instruction::Call(data) => {
                if let Some(name) = func.string_pool.get(data.name_id as usize) {
                    data.name_id = cg.intern(name);
                }
            }
            _ => {}
        }
    }

    let is_expr_body = !matches!(body, AstNode::Block(_, _));
    if is_expr_body && !matches!(func.instructions.last(), Some(Instruction::Return { .. })) {
        func.emit(Instruction::Return {
            value: Some(result_reg),
            loc,
        });
    } else if !matches!(func.instructions.last(), Some(Instruction::Return { .. })) {
        func.emit(Instruction::Return { value: None, loc });
    }

    // Use the shared closure counter for unique naming.
    let closure_name = format!("__closure_{}", cg.closure_counter);
    cg.closure_counter += 1;
    let name_id = cg.intern(&closure_name);
    cg.functions.push(UserFunction {
        name_id,
        params_count: params.len(),
        locals_count: func.next_reg,
        instructions: Arc::from(func.instructions),
    });
    let capture_regs: Vec<usize> = captures
        .iter()
        .filter_map(|name| cg.get_var(name).map(|v| v.idx))
        .collect();
    let dst = cg.alloc_reg();
    cg.emit(Instruction::MakeClosure {
        dst,
        name_id,
        captures: Arc::from(capture_regs),
    });
    Ok(dst)
}
