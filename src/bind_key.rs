use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    ops::{BitXor, BitXorAssign},
};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindKey {
    value: u64,
}

impl BindKey {
    pub const ZERO: Self = Self::new(0);

    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn from_type<T: 'static>() -> Self {
        Self::from_hash(&TypeId::of::<T>())
    }

    pub fn from_hash<T: Hash>(value: &T) -> Self {
        let mut hasher = ahash::AHasher::default();
        value.hash(&mut hasher);
        Self::new(hasher.finish())
    }

    pub const fn value(&self) -> u64 {
        self.value
    }
}

impl BitXor for BindKey {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value ^ rhs.value,
        }
    }
}

impl BitXorAssign for BindKey {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}
