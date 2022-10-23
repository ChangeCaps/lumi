use std::any::TypeId;

use lumi_id::{Id, IdMap};
use lumi_util::HashMap;

pub trait Resource: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Resource for T {}

impl dyn Resource {
    #[inline]
    pub unsafe fn downcast_ref<T: Resource>(&self) -> &T {
        unsafe { &*(self as *const dyn Resource as *const T) }
    }

    #[inline]
    pub unsafe fn downcast_mut<T: Resource>(&mut self) -> &mut T {
        unsafe { &mut *(self as *mut dyn Resource as *mut T) }
    }
}

#[derive(Default)]
pub struct Resources {
    typed: HashMap<TypeId, Box<dyn Resource>>,
}

impl Resources {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn contains<T: Resource>(&self) -> bool {
        self.typed.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn register<T: Resource + Default>(&mut self) {
        if !self.contains::<T>() {
            self.insert(T::default());
        }
    }

    #[inline]
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.typed.insert(TypeId::of::<T>(), Box::new(resource));
    }

    #[inline]
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let resource = self.typed.remove(&TypeId::of::<T>())?;

        Some(unsafe { *Box::from_raw(Box::into_raw(resource) as *mut _) })
    }

    #[inline]
    pub fn remove_or_default<T: Resource + Default>(&mut self) -> T {
        self.remove().unwrap_or_default()
    }

    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let resource = self.typed.get(&TypeId::of::<T>())?;

        Some(unsafe { resource.downcast_ref() })
    }

    #[inline]
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let resource = self.typed.get_mut(&TypeId::of::<T>())?;

        Some(unsafe { resource.downcast_mut() })
    }

    #[inline]
    pub fn get_or_insert_with<T: Resource>(&mut self, f: impl FnOnce() -> T) -> &mut T {
        if !self.contains::<T>() {
            self.insert(f());
        }

        self.get_mut().unwrap()
    }

    #[inline]
    pub fn get_or_default<T: Resource + Default>(&mut self) -> &mut T {
        self.get_or_insert_with(T::default)
    }

    #[inline]
    pub fn scope<Params, Output, Scope: ResourceScope<Params, Output>>(
        &mut self,
        scope: Scope,
    ) -> Output {
        scope.run(self)
    }
}

impl Resources {
    #[inline]
    pub fn id_map<T: Resource>(&self) -> Option<&IdMap<T>> {
        self.get()
    }

    #[inline]
    pub fn id_map_mut<T: Resource>(&mut self) -> &mut IdMap<T> {
        self.get_or_default()
    }

    #[inline]
    pub fn remove_id_map<T: Resource>(&mut self) -> IdMap<T> {
        self.remove_or_default()
    }

    #[inline]
    pub fn contains_id<T: Resource>(&self, id: impl AsRef<Id<T>>) -> bool {
        self.id_map::<T>().map_or(false, |map| map.contains_id(id))
    }

    #[inline]
    pub fn register_id<T: Resource>(&mut self) {
        if !self.contains::<IdMap<T>>() {
            self.insert(IdMap::<T>::new());
        }
    }

    #[inline]
    pub fn insert_id<T: Resource>(&mut self, id: Id<T>, resource: T) {
        self.id_map_mut().insert(id, resource);
    }

    #[inline]
    pub fn remove_id<T: Resource>(&mut self, id: impl AsRef<Id<T>>) -> Option<T> {
        self.id_map_mut().remove(id.as_ref())
    }

    #[inline]
    pub fn get_id<T: Resource>(&self, id: impl AsRef<Id<T>>) -> Option<&T> {
        self.id_map()?.get(id)
    }

    #[inline]
    pub fn get_id_mut<T: Resource>(&mut self, id: impl AsRef<Id<T>>) -> Option<&mut T> {
        self.get_mut::<IdMap<T>>()?.get_mut(id)
    }

    #[inline]
    pub fn get_id_or_insert_with<T: Resource>(
        &mut self,
        id: Id<T>,
        f: impl FnOnce() -> T,
    ) -> &mut T {
        self.id_map_mut().get_or_insert_with(id, f)
    }

    #[inline]
    pub fn get_id_or_default<T: Resource + Default>(&mut self, id: Id<T>) -> &mut T {
        self.get_id_or_insert_with(id, T::default)
    }

    #[inline]
    pub fn iter_id<T: Resource>(&self) -> impl Iterator<Item = (&Id<T>, &T)> {
        self.id_map().into_iter().flat_map(|id_map| id_map.iter())
    }

    #[inline]
    pub fn iter_id_mut<T: Resource>(&mut self) -> impl Iterator<Item = (&Id<T>, &mut T)> {
        self.id_map_mut().iter_mut()
    }

    #[inline]
    pub fn values_id<T: Resource>(&self) -> impl Iterator<Item = &T> {
        self.id_map().into_iter().flat_map(|id_map| id_map.values())
    }

    #[inline]
    pub fn values_id_mut<T: Resource>(&mut self) -> impl Iterator<Item = &mut T> {
        self.id_map_mut().values_mut()
    }

    #[inline]
    pub fn ids<T: Resource>(&self) -> impl Iterator<Item = &Id<T>> {
        self.id_map().into_iter().flat_map(|id_map| id_map.keys())
    }
}

pub trait ResourceScope<Params, Output> {
    fn run(self, resources: &mut Resources) -> Output;
}

macro_rules! impl_resource_scope {
    ($($ident:ident),*) => {
        impl<Func, Output, $($ident: Resource),*> ResourceScope<($($ident,)*), Output> for Func
        where
            Func: FnOnce(&mut Resources, $(&mut $ident),*) -> Output,
        {
            #[allow(unused_variables, non_snake_case)]
            fn run(self, resources: &mut Resources) -> Output {
                $(
                    let mut $ident = resources.remove::<$ident>().expect("Resource not found");
                )*

                let output = (self)(resources, $(&mut $ident),*);

                $(
                    resources.insert($ident);
                )*

                output
            }
        }

        impl_resource_scope!(@ $($ident),*);
    };
    (@ $start:ident $(,$ident:ident)*) => {
        impl_resource_scope!($($ident),*);
    };
    (@) => {}
}

impl_resource_scope!(A, B, C, D, E, F, G, H, I, J, K, L);
