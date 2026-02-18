use std::sync::Arc;

/// Represents a location in the source code.
#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub line: u32,
    pub col: u32,
}

impl From<(usize, usize)> for Loc {
    fn from((line, col): (usize, usize)) -> Self {
        Self {
            line: line as u32,
            col: col as u32,
        }
    }
}

/// A "NaN-Boxed" Value.
///
/// We use the 64-bit space of a double-precision float to store all types.
/// - If the exponent is not all 1s, it's a valid f64.
/// - If it's a NaN, we use the remaining bits to tag it as a Bool or String index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Value(u64);

const QNAN: u64 = 0x7ffc000000000000;
const TAG_BOOL: u64 = 0x0001000000000000;
const TAG_STR: u64 = 0x0002000000000000;
const TAG_LIST: u64 = 0x0003000000000000;

impl Value {
    #[inline(always)]
    pub fn number(n: f64) -> Self {
        Self(n.to_bits())
    }

    #[inline(always)]
    pub fn bool(b: bool) -> Self {
        Self(QNAN | TAG_BOOL | (b as u64))
    }

    #[inline(always)]
    pub fn string(id: u32) -> Self {
        Self(QNAN | TAG_STR | (id as u64))
    }

    #[inline(always)]
    pub fn as_number(self) -> Option<f64> {
        if (self.0 & QNAN) != QNAN {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn as_bool(self) -> Option<bool> {
        if (self.0 & (QNAN | TAG_BOOL)) == (QNAN | TAG_BOOL) {
            Some((self.0 & 1) != 0)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn as_string_id(self) -> Option<u32> {
        if (self.0 & (QNAN | TAG_STR)) == (QNAN | TAG_STR) {
            Some((self.0 & 0xFFFFFFFF) as u32)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn list_id(id: u32) -> Self {
        Self(QNAN | TAG_LIST | (id as u64))
    }

    #[inline(always)]
    pub fn as_list_id(self) -> Option<u32> {
        if (self.0 & (QNAN | TAG_LIST)) == (QNAN | TAG_LIST) {
            Some((self.0 & 0xFFFFFFFF) as u32)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn to_bits(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadLiteral {
        dst: usize,
        val: Value,
    },
    Move {
        dst: usize,
        src: usize,
    },
    LoadGlobal {
        dst: usize,
        global: usize,
    },
    StoreGlobal {
        global: usize,
        src: usize,
    },
    Jump(usize),
    JumpIfFalse {
        cond: usize,
        target: usize,
    },
    Add {
        dst: usize,
        lhs: usize,
        rhs: usize,
        loc: Loc,
    },
    Sub {
        dst: usize,
        lhs: usize,
        rhs: usize,
        loc: Loc,
    },
    Mul {
        dst: usize,
        lhs: usize,
        rhs: usize,
        loc: Loc,
    },
    Div {
        dst: usize,
        lhs: usize,
        rhs: usize,
        loc: Loc,
    },
    Increment(usize),
    LessThan {
        dst: usize,
        lhs: usize,
        rhs: usize,
        loc: Loc,
    },
    Spawn {
        instructions: Arc<[Instruction]>,
        locals_count: usize,
    },
    NewList {
        dst: usize,
        len: usize,
    },
    ListGet {
        dst: usize,
        list: usize,
        index_reg: usize,
        loc: Loc,
    },
    ListSet {
        list: usize,
        index_reg: usize,
        src: usize,
        loc: Loc,
    },
    CallNative {
        name_id: u32,
        args_regs: Arc<[usize]>,
        dst: Option<usize>,
        loc: Loc,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Arc<[Instruction]>,
    pub string_pool: Arc<[Arc<str>]>,
    pub locals_count: usize,
    pub globals_count: usize,
}
