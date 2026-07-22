//! # ys-runtime
//!
//! The ysc execution engine.
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
pub mod value_fmt;
pub mod vm;
pub mod yatsu;

//  Public re-exports

pub use context::{Backend, Callable, Context, NativeFn};
pub use heap::{Generation, Heap, HeapMetadata, HeapObject, ManagedObject, ObjectData};
pub use value_ext::ValueExt;
pub use value_fmt::stringify_value;
pub use vm::{Interpreter, run_interpreter};
