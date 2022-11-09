use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
};

use shiv::storage::Resource;

pub struct OwnedPtr<T: Resource> {
    ptr: Arc<AtomicPtr<T>>,
}

impl<T: Resource> OwnedPtr<T> {
    #[inline]
    pub fn get_ptr(&self) -> Option<*const T> {
        let ptr = self.ptr.load(Ordering::Acquire);

        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    #[inline]
    pub fn ptr(&self) -> *const T {
        self.get_ptr()
            .expect("ResourcePtr used after ResourceGuard was dropped")
    }
}

impl<T: Resource> Deref for OwnedPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: the pointer is only null if the resource is not present.
        unsafe { &*self.ptr() }
    }
}

impl<T: Resource> Clone for OwnedPtr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T: Resource> std::fmt::Debug for OwnedPtr<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedPtr")
            .field("ptr", &self.get_ptr())
            .finish()
    }
}

pub struct OwnedPtrGuard<'a, T: Resource> {
    ptr: Arc<AtomicPtr<T>>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: Resource> OwnedPtrGuard<'a, T> {
    #[inline]
    pub fn new(resource: &T) -> Self {
        Self {
            ptr: Arc::new(AtomicPtr::new(resource as *const T as *mut T)),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn get(self: Pin<&mut Self>) -> OwnedPtr<T> {
        OwnedPtr {
            ptr: self.ptr.clone(),
        }
    }
}

impl<'a, T: Resource> Drop for OwnedPtrGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.ptr.store(ptr::null_mut(), Ordering::Release)
    }
}

pub struct OwnedPtrMut<T: Resource> {
    ptr: Arc<AtomicPtr<T>>,
}

impl<T: Resource> OwnedPtrMut<T> {
    #[inline]
    pub fn get_ptr(&self) -> Option<*mut T> {
        let ptr = self.ptr.load(Ordering::Acquire);

        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }

    #[inline]
    pub fn ptr(&self) -> *mut T {
        self.get_ptr()
            .expect("ResourcePtr used after ResourceGuard was dropped")
    }
}

impl<T: Resource> Deref for OwnedPtrMut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: the pointer is only null if the resource is not present.
        unsafe { &*self.ptr() }
    }
}

impl<T: Resource> DerefMut for OwnedPtrMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: the pointer is only null if the resource is not present.
        unsafe { &mut *self.ptr() }
    }
}

impl<T: Resource> std::fmt::Debug for OwnedPtrMut<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedPtrMut")
            .field("ptr", &self.get_ptr())
            .finish()
    }
}

pub struct OwnedPtrMutGuard<'a, T: Resource> {
    ptr: Arc<AtomicPtr<T>>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: Resource> OwnedPtrMutGuard<'a, T> {
    #[inline]
    pub fn new(resource: &mut T) -> Self {
        Self {
            ptr: Arc::new(AtomicPtr::new(resource)),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn get(self: Pin<&mut Self>) -> OwnedPtrMut<T> {
        OwnedPtrMut {
            ptr: self.ptr.clone(),
        }
    }
}

impl<'a, T: Resource> Drop for OwnedPtrMutGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.ptr.store(ptr::null_mut(), Ordering::Release)
    }
}

#[macro_export]
macro_rules! guard {
    ($ident:ident) => {
        let guard = $crate::OwnedPtrGuard::new($ident);
        ::futures_lite::pin!(guard);

        #[allow(unused_mut)]
        let mut $ident = guard.get();
    };
}

#[macro_export]
macro_rules! guard_mut {
    ($ident:ident) => {
        let guard = $crate::OwnedPtrMutGuard::new($ident);
        ::futures_lite::pin!(guard);

        #[allow(unused_mut)]
        let mut $ident = guard.get();
    };
}
