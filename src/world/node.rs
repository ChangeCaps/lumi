use std::any::TypeId;

pub unsafe trait Node: Send + Sync + 'static {
    fn type_id(&self) -> TypeId;
}
unsafe impl<T: Send + Sync + 'static> Node for T {
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Node {
    pub unsafe fn downcast_ref_unchecked<T: Node>(&self) -> &T {
        unsafe { &*(self as *const dyn Node as *const T) }
    }

    pub unsafe fn downcast_mut_unchecked<T: Node>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Node as *mut T) }
    }

    pub fn downcast_ref<T: Node>(&self) -> Option<&T> {
        if Node::type_id(self) == TypeId::of::<T>() {
            Some(unsafe { &*(self as *const dyn Node as *const T) })
        } else {
            None
        }
    }

    pub fn downcast_mut<T: Node>(&mut self) -> Option<&mut T> {
        if Node::type_id(self) == TypeId::of::<T>() {
            Some(unsafe { &mut *(self as *mut dyn Node as *mut T) })
        } else {
            None
        }
    }
}
