//! Expression code generation — template literals, function calls, short-circuit
//! operators.
//!
//! These are free functions that take `&mut Codegen` so they can be defined in
//! a child module without creating a circular dependency.

use super::Codegen;
use crate::ast::*;
use crate::compiler::*;
use crate::error::JitError;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Template literals
// ---------------------------------------------------------------------------

pub(super) fn compile_template(
    cg: &mut Codegen,
    parts: &[TemplatePart],
    loc: Loc,
) -> Result<usize, JitError> {
    let mut result: Option<usize> = None;
    for part in parts {
        match part {
            TemplatePart::Text(s) => {
                let r = cg.alloc_reg();
                let val = Value::sso(s).unwrap_or_else(|| Value::pool(cg.intern(s)));
                cg.emit(Instruction::LoadLiteral { dst: r, val, loc });
                result = Some(concat(cg, result, r, loc)?);
            }
            TemplatePart::Expr(expr) => {
                let r = cg.compile_node(expr)?;
                // Wrap in str() call to ensure string
                let str_dst = cg.alloc_reg();
                let str_name = cg.intern("str");
                cg.emit(Instruction::Call(CallData {
                    name_id: str_name,
                    args_regs: Arc::from(vec![r]),
                    dst: Some(str_dst),
                    loc,
                }));
                result = Some(concat(cg, result, str_dst, loc)?);
            }
        }
    }
    Ok(result.unwrap_or_else(|| {
        let dst = cg.alloc_reg();
        cg.emit(Instruction::LoadLiteral {
            dst,
            val: Value::nil(),
            loc,
        });
        dst
    }))
}

pub(super) fn concat(
    cg: &mut Codegen,
    left: Option<usize>,
    right: usize,
    loc: Loc,
) -> Result<usize, JitError> {
    match left {
        None => Ok(right),
        Some(l) => {
            let dst = cg.alloc_reg();
            cg.emit(Instruction::Add {
                dst,
                lhs: l,
                rhs: right,
                loc,
            });
            cg.free_reg(l);
            cg.free_reg(right);
            Ok(dst)
        }
    }
}

// ---------------------------------------------------------------------------
// Function calls
// ---------------------------------------------------------------------------

pub(super) fn compile_fun_call(
    cg: &mut Codegen,
    name: &str,
    args: &[AstNode],
    loc: Loc,
) -> Result<usize, JitError> {
    let args_r: Vec<usize> = args
        .iter()
        .map(|a| cg.compile_node(a))
        .collect::<Result<_, _>>()?;
    let dst = cg.alloc_reg();
    if let Some(info) = cg.get_var(name) {
        // Variable holding a callable — dynamic dispatch
        let callee_reg = cg.load_var(info);
        for &r in &args_r {
            cg.free_reg(r);
        }
        if info.is_global {
            cg.free_reg(callee_reg);
        }
        cg.emit(Instruction::CallDynamic(CallDynamicData {
            callee_reg,
            args_regs: Arc::from(args_r),
            dst: Some(dst),
            loc,
        }));
    } else {
        for &r in &args_r {
            cg.free_reg(r);
        }
        let name_id = cg.intern(name);
        cg.emit(Instruction::Call(CallData {
            name_id,
            args_regs: Arc::from(args_r),
            dst: Some(dst),
            loc,
        }));
    }
    Ok(dst)
}

pub(super) fn compile_args(cg: &mut Codegen, args: &[AstNode]) -> Result<Vec<usize>, JitError> {
    args.iter().map(|a| cg.compile_node(a)).collect()
}

// ---------------------------------------------------------------------------
// Short-circuit && and ||
// ---------------------------------------------------------------------------

pub(super) fn compile_short_circuit(
    cg: &mut Codegen,
    op: BinOp,
    l: usize,
    r: usize,
    _loc: Loc,
) -> Result<usize, JitError> {
    let dst = cg.alloc_reg();
    match op {
        BinOp::And => {
            // a && b: if a is falsy → short-circuit (result = a), else evaluate b
            cg.emit(Instruction::Move { dst, src: l });
            let jump_idx = cg.instructions.len();
            cg.emit(Instruction::Jump(0)); // placeholder
            cg.emit(Instruction::Move { dst, src: r });
            let end = cg.instructions.len();
            cg.instructions[jump_idx] = Instruction::JumpIfFalse {
                cond: l,
                target: end,
            };
        }
        BinOp::Or => {
            // a || b: if a is truthy → short-circuit (result = a), else evaluate b
            // Only have JumpIfFalse, so invert: if l is falsy, evaluate r
            cg.emit(Instruction::Move { dst, src: l });
            let jump_false_idx = cg.instructions.len();
            cg.emit(Instruction::Jump(0)); // placeholder → JumpIfFalse to eval_r
            let jump_end_idx = cg.instructions.len();
            cg.emit(Instruction::Jump(0)); // placeholder → Jump(end) when truthy
            let eval_r = cg.instructions.len();
            cg.emit(Instruction::Move { dst, src: r });
            let end = cg.instructions.len();
            cg.instructions[jump_false_idx] = Instruction::JumpIfFalse {
                cond: l,
                target: eval_r,
            };
            cg.instructions[jump_end_idx] = Instruction::Jump(end);
        }
        _ => unreachable!(),
    }
    Ok(dst)
}
