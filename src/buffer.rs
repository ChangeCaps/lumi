use bytemuck::{AnyBitPattern, Pod};
use wgpu::BufferUsages;

use crate::{id::BufferId, SharedDevice, SharedQueue};

use std::{
    alloc, mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Debug)]
pub struct SharedBuffer {
    buffer: Arc<wgpu::Buffer>,
    id: BufferId,
}

impl SharedBuffer {
    pub fn new(buffer: wgpu::Buffer) -> Self {
        Self {
            buffer: Arc::new(buffer),
            id: BufferId::new(),
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn id(&self) -> BufferId {
        self.id
    }
}

impl Deref for SharedBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        self.buffer()
    }
}

impl PartialEq for SharedBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for SharedBuffer {}

pub struct VecBuffer {
    data: NonNull<u8>,
    size: usize,
    mutated: Mutex<bool>,
    buffer: SharedBuffer,
    device: SharedDevice,
    queue: SharedQueue,
    usage: BufferUsages,
}

impl VecBuffer {
    const ALIGN: usize = 16;

    fn layout(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(self.size, Self::ALIGN) }
    }

    fn mark_mutated(&mut self) {
        if let Ok(x) = self.mutated.get_mut() {
            *x = true;
        }
    }

    pub fn new(
        device: &SharedDevice,
        queue: &SharedQueue,
        size: usize,
        usage: BufferUsages,
    ) -> Self {
        let layout = unsafe { alloc::Layout::from_size_align_unchecked(size, Self::ALIGN) };
        let ptr = unsafe { alloc::alloc(layout) };

        let data = match NonNull::new(ptr) {
            Some(data) => data,
            None => alloc::handle_alloc_error(layout),
        };

        let buffer = device.create_shared_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as u64,
            usage,
            mapped_at_creation: false,
        });

        Self {
            data,
            size,
            mutated: Mutex::new(false),
            buffer,
            device: device.clone(),
            queue: queue.clone(),
            usage,
        }
    }

    pub fn with_data(
        device: &SharedDevice,
        queue: &SharedQueue,
        data: &[u8],
        usage: BufferUsages,
    ) -> Self {
        let mut this = Self::new(device, queue, data.len(), usage);

        this.copy_from_slice(data);

        this
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn resize(&mut self, new_size: usize) {
        let new_ptr = unsafe { alloc::realloc(self.data.as_ptr(), self.layout(), new_size) };

        let new_data = match NonNull::new(new_ptr) {
            Some(data) => data,
            None => alloc::handle_alloc_error(self.layout()),
        };

        self.data = new_data;
        self.size = new_size;

        let new_buffer = self.device.create_shared_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: self.size as u64,
            mapped_at_creation: false,
            usage: self.usage,
        });

        self.buffer = new_buffer;

        self.mark_mutated();
    }

    pub fn buffer(&self) -> &SharedBuffer {
        let mut mutated = self.mutated.lock().unwrap();

        if *mutated {
            self.queue.write_buffer(self.buffer(), 0, self);

            *mutated = false;
        }

        &self.buffer
    }

    pub fn get<T: AnyBitPattern>(&self, offset: usize) -> Option<&T> {
        if offset <= self.size - mem::size_of::<T>() {
            return None;
        }

        let ptr = unsafe { self.data.as_ptr().offset(offset as isize) };
        assert!(
            ptr.align_offset(mem::align_of::<T>()) == 0,
            "offset must be aligned to mem::align_of::<T>()"
        );

        Some(unsafe { &*(ptr as *const T) })
    }

    pub fn get_mut<T: Pod>(&mut self, offset: usize) -> Option<&mut T> {
        if offset <= self.size - mem::size_of::<T>() {
            return None;
        }

        let ptr = unsafe { self.data.as_ptr().offset(offset as isize) };
        assert!(
            ptr.align_offset(mem::align_of::<T>()) == 0,
            "offset must be aligned to mem::align_of::<T>()"
        );

        self.mark_mutated();
        Some(unsafe { &mut *(ptr as *mut T) })
    }

    pub fn cast<T: AnyBitPattern>(&self) -> &[T] {
        let len = self.size / mem::size_of::<T>();

        bytemuck::cast_slice(&self.deref()[..mem::size_of::<T>() * len])
    }

    pub fn cast_mut<T: Pod>(&mut self) -> &mut [T] {
        let len = self.size / mem::size_of::<T>();

        self.mark_mutated();
        bytemuck::cast_slice_mut(&mut self.deref_mut()[..mem::size_of::<T>() * len])
    }
}

impl Clone for VecBuffer {
    fn clone(&self) -> Self {
        Self::with_data(&self.device, &self.queue, self, self.usage)
    }
}

impl Deref for VecBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.size) }
    }
}

impl DerefMut for VecBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mark_mutated();
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.size) }
    }
}

impl Drop for VecBuffer {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { alloc::dealloc(self.data.as_ptr(), self.layout()) };
        }
    }
}
