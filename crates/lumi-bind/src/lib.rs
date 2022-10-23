#![deny(unsafe_op_in_unsafe_fn)]

mod binding;

pub use binding::*;
pub use lumi_macro::Bind;

use std::collections::LinkedList;

use lumi_core::{BindKey, BindingLayoutEntry, Device, Queue};

pub trait Bind {
    fn entries() -> LinkedList<BindingLayoutEntry>
    where
        Self: Sized;

    fn bind_key(&self) -> BindKey;

    fn bind(&self, device: &Device, queue: &Queue, bindings: &mut Bindings);
}
