use std::{any::TypeId, collections::HashMap};

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

impl dyn Resource {
    pub unsafe fn downcast_ref<T: Resource>(&self) -> &T {
        unsafe { &*(self as *const dyn Resource as *const T) }
    }

    pub unsafe fn downcast_mut<T: Resource>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Resource as *mut T) }
    }
}

#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, Box<dyn Resource>>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource = self.resources.remove(&TypeId::of::<T>())?;

        Some(unsafe { *Box::from_raw(Box::into_raw(resource) as *mut _) })
    }

    pub fn get<T: Resource>(&self) -> Option<&T> {
        let resource = self.resources.get(&TypeId::of::<T>())?;

        Some(unsafe { &*(resource.as_ref() as *const dyn Resource as *const T) })
    }

    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let resource = self.resources.get_mut(&TypeId::of::<T>())?;

        Some(unsafe { &mut *(resource.as_mut() as *mut dyn Resource as *mut T) })
    }

    pub fn get_or_default<T: Resource + Default>(&mut self) -> &T {
        self.get_mut_or_default()
    }

    pub fn get_mut_or_default<T: Resource + Default>(&mut self) -> &mut T {
        if !self.contains::<T>() {
            self.insert(T::default());
        }

        self.get_mut().unwrap()
    }
}
