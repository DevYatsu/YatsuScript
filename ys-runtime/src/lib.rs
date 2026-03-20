//! # ys-runtime
//!
//! The YatsuScript execution engine.
//!
//! Provides:
//! - The [`Backend`] trait and [`Interpreter`] implementation
//! - The [`Context`] shared execution state (globals, string pool, callables)
//! - The [`Heap`] generational garbage-collected object store
//! - Native built-in functions (`print`, `len`, `str`, `sleep`, ...)
//! - [`ValueExt`] — heap-dependent extension methods on [`Value`]

pub mod context;
pub mod heap;
pub mod natives;
pub mod value_ext;
pub mod vm;
pub mod value_fmt;

//  Public re-exports 

pub use context::{Backend, Callable, Context, NativeFn};
pub use heap::{Generation, Heap, HeapMetadata, HeapObject, ManagedObject};
pub use value_ext::ValueExt;
pub use vm::{Interpreter, run_interpreter};
pub use value_fmt::stringify_value;
