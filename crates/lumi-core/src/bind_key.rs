use std::{
    any::TypeId,
    hash::Hash,
    ops::{BitXor, BitXorAssign},
};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindKey {
    value: u64,
}

impl BindKey {
    pub const ZERO: Self = Self::new(0);

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    #[inline]
    pub fn from_type<T: 'static>() -> Self {
        Self::from_hash(&TypeId::of::<T>())
    }

    #[inline]
    pub fn from_hash<T: Hash>(value: T) -> Self {
        Self {
            value: lumi_util::hash(value),
        }
    }

    #[inline]
    pub const fn value(&self) -> u64 {
        self.value
    }
}

impl BitXor for BindKey {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

impl BitXorAssign for BindKey {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}
