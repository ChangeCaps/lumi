use std::any::TypeId;

use wgpu::{CommandEncoder, Device, Queue};

use crate::{frame_buffer::FrameBuffer, prelude::World, resources::Resources, world::WorldChanges};

use super::{PhaseLabel, View};

#[derive(Clone, Copy, Debug)]
pub struct ViewPhaseContext<'a> {
    pub label: PhaseLabel,
    pub changes: &'a WorldChanges,
    pub view: &'a View,
    pub device: &'a Device,
    pub queue: &'a Queue,
}

#[allow(unused_variables)]
pub trait RenderViewPhase: Send + Sync + 'static {
    fn prepare(
        &mut self,
        context: &ViewPhaseContext,
        target: &FrameBuffer,
        world: &World,
        resources: &mut Resources,
    );
    fn render(
        &self,
        context: &ViewPhaseContext,
        encoder: &mut CommandEncoder,
        target: &FrameBuffer,
        world: &World,
        resources: &Resources,
    );
}

struct ViewPhaseEntry {
    label: PhaseLabel,
    phase: Box<dyn RenderViewPhase>,
}

#[derive(Default)]
pub struct RenderViewPhases {
    phases: Vec<ViewPhaseEntry>,
}

impl RenderViewPhases {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push<T: RenderViewPhase>(&mut self, label: impl Into<PhaseLabel>, phase: T) {
        self.phases.push(ViewPhaseEntry {
            label: label.into(),
            phase: Box::new(phase),
        });
    }

    pub fn index_of(&self, label: impl Into<PhaseLabel>) -> Option<usize> {
        let label = label.into();
        self.phases.iter().position(|entry| entry.label == label)
    }

    #[track_caller]
    pub fn insert_before<T: RenderViewPhase>(
        &mut self,
        before: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) {
        let before = before.into();
        let index = self.index_of(before).expect("Phase not found");
        self.phases.insert(
            index,
            ViewPhaseEntry {
                label: label.into(),
                phase: Box::new(phase),
            },
        );
    }

    #[track_caller]
    pub fn insert_after<T: RenderViewPhase>(
        &mut self,
        after: impl Into<PhaseLabel>,
        label: impl Into<PhaseLabel>,
        phase: T,
    ) {
        let after = after.into();
        let index = self.index_of(after).expect("Phase not found");
        self.phases.insert(
            index + 1,
            ViewPhaseEntry {
                label: label.into(),
                phase: Box::new(phase),
            },
        );
    }

    fn get_entry(&self, label: impl Into<PhaseLabel>) -> Option<&ViewPhaseEntry> {
        let label = label.into();
        self.phases.iter().find(|entry| entry.label == label)
    }

    fn get_entry_mut(&mut self, label: impl Into<PhaseLabel>) -> Option<&mut ViewPhaseEntry> {
        let label = label.into();
        self.phases.iter_mut().find(|entry| entry.label == label)
    }

    pub fn get_phase(&self, label: impl Into<PhaseLabel>) -> Option<&dyn RenderViewPhase> {
        self.get_entry(label).map(|entry| entry.phase.as_ref())
    }

    pub fn get_phase_mut(
        &mut self,
        label: impl Into<PhaseLabel>,
    ) -> Option<&mut dyn RenderViewPhase> {
        self.get_entry_mut(label).map(|entry| entry.phase.as_mut())
    }

    pub fn get<T: RenderViewPhase>(&self, label: impl Into<PhaseLabel>) -> Option<&T> {
        self.get_entry(label).and_then(|entry| {
            if entry.label.type_id() == TypeId::of::<T>() {
                Some(unsafe { &*(entry.phase.as_ref() as *const dyn RenderViewPhase as *const T) })
            } else {
                None
            }
        })
    }

    pub fn get_mut<T: RenderViewPhase>(&mut self, label: impl Into<PhaseLabel>) -> Option<&mut T> {
        self.get_entry_mut(label).and_then(|entry| {
            if entry.label.type_id() == TypeId::of::<T>() {
                Some(unsafe { &mut *(entry.phase.as_mut() as *mut dyn RenderViewPhase as *mut T) })
            } else {
                None
            }
        })
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        target: &FrameBuffer,
        view: &View,
        world: &World,
        changes: &WorldChanges,
        resources: &mut Resources,
    ) {
        for entry in &mut self.phases {
            let context = ViewPhaseContext {
                label: entry.label,
                changes,
                view,
                device,
                queue,
            };

            entry.phase.prepare(&context, target, world, resources);
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        target: &FrameBuffer,
        view: &View,
        world: &World,
        changes: &WorldChanges,
        resources: &Resources,
    ) {
        for entry in &self.phases {
            let context = ViewPhaseContext {
                label: entry.label,
                changes,
                view,
                device,
                queue,
            };

            entry
                .phase
                .render(&context, encoder, target, world, resources);
        }
    }
}
