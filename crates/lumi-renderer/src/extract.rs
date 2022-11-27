use std::ops::{Deref, DerefMut};

use shiv::{
    query::{Query, With},
    system::{
        Commands, ReadOnlySystemParamFetch, ResState, SystemMeta, SystemParam, SystemParamFetch,
        SystemParamItem, SystemParamState, SystemState,
    },
    world::{Component, Entity, World},
};

use crate::OwnedPtrMut;

pub type MainWorld = OwnedPtrMut<World>;

#[derive(Clone, Copy, Debug)]
pub struct DespawnExtracted {
    pub entity: Entity,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Extracted;

impl Extracted {
    pub fn spawn_system(
        mut commands: Commands,
        extract_query: Extract<Query<Entity>>,
        extracted_query: Query<(), With<Extracted>>,
    ) {
        for entity in extract_query.iter() {
            if !extracted_query.contains(entity) {
                commands.get_or_spawn(entity).insert(Extracted);
            }
        }
    }

    pub fn despawn_system(
        mut commands: Commands,
        extract_query: Extract<Query<()>>,
        extracted_query: Query<Entity, With<Extracted>>,
    ) {
        for entity in extracted_query.iter() {
            if !extract_query.contains(entity) {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub struct Extract<'w, 's, P: SystemParam>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    item: <P::Fetch as SystemParamFetch<'w, 's>>::Item,
}

impl<'w, 's, P: SystemParam + 'static> SystemParam for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    type Fetch = ExtractState<P>;
}

#[doc(hidden)]
pub struct ExtractState<P: SystemParam + 'static> {
    state: SystemState<P>,
    main_world_state: ResState<MainWorld>,
}

unsafe impl<P: SystemParam + 'static> SystemParamState for ExtractState<P> {
    fn init(world: &mut World, meta: &mut SystemMeta) -> Self {
        let mut main_world = world.resource_mut::<MainWorld>();

        Self {
            state: SystemState::new(&mut main_world),
            main_world_state: ResState::init(world, meta),
        }
    }
}

impl<'w, 's, P: SystemParam + 'static> SystemParamFetch<'w, 's> for ExtractState<P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    type Item = Extract<'w, 's, P>;

    unsafe fn get_param(
        &'s mut self,
        meta: &SystemMeta,
        world: &'w World,
        change_tick: u32,
    ) -> Self::Item {
        let main_world = unsafe { self.main_world_state.get_param(meta, world, change_tick) };
        let item = self.state.get(main_world.into_inner());

        Extract { item }
    }
}

impl<'w, 's, P: SystemParam> Deref for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    type Target = SystemParamItem<'w, 's, P>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'w, 's, P: SystemParam> DerefMut for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<'w, 's, P: SystemParam> IntoIterator for Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
    SystemParamItem<'w, 's, P>: IntoIterator,
{
    type Item = <SystemParamItem<'w, 's, P> as IntoIterator>::Item;
    type IntoIter = <SystemParamItem<'w, 's, P> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.item.into_iter()
    }
}

impl<'a, 'w, 's, P: SystemParam> IntoIterator for &'a Extract<'w, 's, P>
where
    P::Fetch: ReadOnlySystemParamFetch,
    &'a SystemParamItem<'w, 's, P>: IntoIterator,
{
    type Item = <&'a SystemParamItem<'w, 's, P> as IntoIterator>::Item;
    type IntoIter = <&'a SystemParamItem<'w, 's, P> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.item).into_iter()
    }
}
