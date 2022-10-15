use std::any::TypeId;

use wgpu::{CommandEncoder, Device, Queue};

use crate::{
    prelude::{Resources, World},
    world::WorldChanges,
};

pub use lumi_macro::PhaseLabel;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhaseLabel {
    type_id: TypeId,
    variant: usize,
}

impl PhaseLabel {
    pub fn new<T: 'static>(variant: usize) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            variant,
        }
    }

    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub const fn variant(&self) -> usize {
        self.variant
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PhaseLabel)]
pub enum DefaultPhases {
    PrepareMeshes,
    PrepareLights,
    Shadow,
    Material,
}

#[derive(Clone, Copy, Debug)]
pub struct PhaseContext<'a> {
    pub label: PhaseLabel,
    pub changes: &'a WorldChanges,
    pub device: &'a Device,
    pub queue: &'a Queue,
}

#[allow(unused_variables)]
pub trait RenderPhase: Send + Sync + 'static {
    fn prepare(&mut self, context: &PhaseContext, world: &World, resources: &mut Resources) {}
    fn render(
        &self,
        context: &PhaseContext,
        encoder: &mut CommandEncoder,
        world: &World,
        resources: &Resources,
    ) {
    }
}

struct PhaseEntry {
    label: PhaseLabel,
    phase: Box<dyn RenderPhase>,
}

#[derive(Default)]
pub struct RenderPhases {
    phases: Vec<PhaseEntry>,
}

impl RenderPhases {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<T: RenderPhase>(&mut self, label: impl Into<PhaseLabel>, phase: T) {
        self.phases.push(PhaseEntry {
            label: label.into(),
            phase: Box::new(phase),
        });
    }

    pub fn index_of(&self, label: impl Into<PhaseLabel>) -> Option<usize> {
        let label = label.into();
        self.phases.iter().position(|entry| entry.label == label)
    }

    #[track_caller]
    pub fn insert_before<T: RenderPhase>(
        &mut self,
        before: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) {
        let before = before.into();
        let index = self.index_of(before).expect("Phase not found");
        self.phases.insert(
            index,
            PhaseEntry {
                label: label.into(),
                phase: Box::new(phase),
            },
        );
    }

    #[track_caller]
    pub fn insert_after<T: RenderPhase>(
        &mut self,
        after: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) {
        let after = after.into();
        let index = self.index_of(after).expect("Phase not found");
        self.phases.insert(
            index + 1,
            PhaseEntry {
                label: label.into(),
                phase: Box::new(phase),
            },
        );
    }

    fn get_entry(&self, label: impl Into<PhaseLabel>) -> Option<&PhaseEntry> {
        let label = label.into();
        self.phases.iter().find(|entry| entry.label == label)
    }

    fn get_entry_mut(&mut self, label: impl Into<PhaseLabel>) -> Option<&mut PhaseEntry> {
        let label = label.into();
        self.phases.iter_mut().find(|entry| entry.label == label)
    }

    pub fn get_phase(&self, label: impl Into<PhaseLabel>) -> Option<&dyn RenderPhase> {
        self.get_entry(label).map(|entry| entry.phase.as_ref())
    }

    pub fn get_phase_mut(&mut self, label: impl Into<PhaseLabel>) -> Option<&mut dyn RenderPhase> {
        self.get_entry_mut(label).map(|entry| entry.phase.as_mut())
    }

    pub fn get<T: RenderPhase>(&self, label: impl Into<PhaseLabel>) -> Option<&T> {
        self.get_entry(label).and_then(|entry| {
            if entry.label.type_id() == TypeId::of::<T>() {
                Some(unsafe { &*(entry.phase.as_ref() as *const dyn RenderPhase as *const T) })
            } else {
                None
            }
        })
    }

    pub fn get_mut<T: RenderPhase>(&mut self, label: impl Into<PhaseLabel>) -> Option<&mut T> {
        self.get_entry_mut(label).and_then(|entry| {
            if entry.label.type_id() == TypeId::of::<T>() {
                Some(unsafe { &mut *(entry.phase.as_mut() as *mut dyn RenderPhase as *mut T) })
            } else {
                None
            }
        })
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        world: &World,
        changes: &WorldChanges,
        resources: &mut Resources,
    ) {
        for entry in self.phases.iter_mut() {
            let context = PhaseContext {
                label: entry.label,
                changes,
                device,
                queue,
            };

            entry.phase.prepare(&context, world, resources);
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        world: &World,
        changes: &WorldChanges,
        resources: &Resources,
    ) {
        for entry in self.phases.iter() {
            let context = PhaseContext {
                label: entry.label,
                changes,
                device,
                queue,
            };

            entry.phase.render(&context, encoder, world, resources);
        }
    }
}
