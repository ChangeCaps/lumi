use std::{
    any::Any,
    borrow::Cow,
    collections::{HashMap, LinkedList},
};

use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, RenderPass,
};

use crate::{
    bind::{Bind, BindingLayoutEntry, SharedBindingResource},
    shader::Shader,
    SharedDevice, SharedQueue,
};

#[derive(Debug)]
pub struct BindingLocation {
    pub binding: u32,
    pub group: u32,
}

#[derive(Debug)]
pub struct BindingsLayout {
    entries: LinkedList<BindingLayoutEntry>,
    bindings: HashMap<Cow<'static, str>, BindingLocation>,
}

impl BindingsLayout {
    pub fn new() -> Self {
        Self {
            entries: LinkedList::new(),
            bindings: HashMap::new(),
        }
    }

    pub fn with_shader(mut self, shader: &Shader) -> Self {
        for (_, variable) in shader.module().global_variables.iter() {
            if let Some(ref name) = variable.name {
                if let Some(ref binding) = variable.binding {
                    let binding = BindingLocation {
                        binding: binding.binding,
                        group: binding.group,
                    };

                    self.add_binding(name.clone(), binding);
                }
            }
        }

        self
    }

    pub fn add_binding(&mut self, name: impl Into<Cow<'static, str>>, binding: BindingLocation) {
        self.bindings.insert(name.into(), binding);
    }

    pub fn bind<T: Bind>(self) -> Self {
        self.append(T::entries())
    }

    pub fn push(mut self, entry: BindingLayoutEntry) -> Self {
        self.entries.push_back(entry);
        self
    }

    pub fn append(mut self, entries: LinkedList<BindingLayoutEntry>) -> Self {
        for entry in entries {
            if self.bindings.contains_key(&entry.name) {
                self.entries.push_back(entry);
            }
        }
        self
    }

    pub fn create_bind_group_layouts(&self, device: &SharedDevice) -> Vec<BindGroupLayout> {
        let mut entries: Vec<Vec<BindGroupLayoutEntry>> = Vec::new();

        for entry in self.entries.iter() {
            let binding = self.bindings.get(&entry.name).unwrap();

            while entries.len() <= binding.group as usize {
                entries.push(Vec::new());
            }

            let entries = &mut entries[binding.group as usize];

            let entry = BindGroupLayoutEntry {
                binding: binding.binding,
                visibility: entry.visibility,
                ty: entry.ty,
                count: entry.count,
            };

            entries.push(entry);
        }

        entries
            .into_iter()
            .map(|entries| {
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &entries,
                })
            })
            .collect()
    }

    pub fn create_bindings(&self, device: &SharedDevice) -> Bindings {
        let mut groups: Vec<BindingGroup> = self
            .create_bind_group_layouts(device)
            .into_iter()
            .map(|layout| BindingGroup {
                bindings: HashMap::new(),
                layout,
                bind_group: None,
            })
            .collect();

        for entry in self.entries.iter() {
            let binding = self.bindings.get(&entry.name).unwrap();

            let group_entry = BindingGroupEntry::new(binding.binding, (entry.state)());

            let group = &mut groups[binding.group as usize];
            let key = BindingGroupKey::new(entry.name.clone(), entry.ty);
            group.bindings.insert(key, group_entry);
        }

        Bindings { groups }
    }
}

struct BindingGroupEntry {
    resource: Option<SharedBindingResource>,
    binding: u32,
    state: Box<dyn Any + Send + Sync>,
}

impl BindingGroupEntry {
    pub const fn new(binding: u32, state: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            resource: None,
            binding,
            state,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum BindingKind {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct BindingGroupKey {
    kind: BindingKind,
    name: Cow<'static, str>,
}

impl BindingGroupKey {
    const fn new(name: Cow<'static, str>, ty: BindingType) -> Self {
        let kind = match ty {
            BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                ..
            } => BindingKind::UniformBuffer,
            BindingType::Buffer {
                ty: BufferBindingType::Storage { .. },
                ..
            } => BindingKind::StorageBuffer,
            BindingType::Texture { .. } => BindingKind::Texture,
            BindingType::StorageTexture { .. } => BindingKind::StorageTexture,
            BindingType::Sampler { .. } => BindingKind::Sampler,
        };

        Self { name, kind }
    }
}

struct BindingGroup {
    bindings: HashMap<BindingGroupKey, BindingGroupEntry>,
    layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
}

impl BindingGroup {
    #[track_caller]
    pub fn create_bind_group(&self, device: &SharedDevice) -> BindGroup {
        let mut entries = Vec::with_capacity(self.bindings.len());

        for (name, entry) in self.bindings.iter() {
            let resource = entry
                .resource
                .as_ref()
                .unwrap_or_else(|| panic!("Binding '{}' not bound", name.name));

            let entry = wgpu::BindGroupEntry {
                binding: entry.binding,
                resource: resource.as_binding_resource(),
            };

            entries.push(entry);
        }

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &entries,
        })
    }
}

pub struct Bindings {
    groups: Vec<BindingGroup>,
}

impl Bindings {
    pub fn new(device: &SharedDevice, layout: &BindingsLayout) -> Self {
        layout.create_bindings(device)
    }

    pub fn layouts(&self) -> Vec<&BindGroupLayout> {
        self.groups.iter().map(|g| &g.layout).collect()
    }

    #[track_caller]
    pub fn bind<T>(&mut self, device: &SharedDevice, queue: &SharedQueue, bind: &T)
    where
        T: Bind + ?Sized,
    {
        for group in self.groups.iter_mut() {
            for (key, entry) in group.bindings.iter_mut() {
                let resource = match key.kind {
                    BindingKind::UniformBuffer => {
                        bind.get_uniform(device, queue, key.name.as_ref(), entry.state.as_mut())
                    }
                    BindingKind::StorageBuffer => {
                        bind.get_storage(device, queue, key.name.as_ref(), entry.state.as_mut())
                    }
                    BindingKind::Texture => {
                        bind.get_texture(device, queue, key.name.as_ref(), entry.state.as_mut())
                    }
                    BindingKind::StorageTexture => bind.get_storage_texture(
                        device,
                        queue,
                        key.name.as_ref(),
                        entry.state.as_mut(),
                    ),
                    BindingKind::Sampler => {
                        bind.get_sampler(device, queue, key.name.as_ref(), entry.state.as_mut())
                    }
                };

                if resource.is_some() {
                    if entry.resource != resource {
                        group.bind_group = None;
                    }

                    entry.resource = resource;
                }
            }
        }
    }

    pub fn update_bind_groups(&mut self, device: &SharedDevice) {
        for group in self.groups.iter_mut() {
            if group.bind_group.is_none() {
                group.bind_group = Some(group.create_bind_group(device));
            }
        }
    }

    #[track_caller]
    pub fn bind_groups(&self) -> impl Iterator<Item = &BindGroup> {
        self.groups
            .iter()
            .map(|g| g.bind_group.as_ref().expect("BindGroup not created"))
    }

    pub fn bind_pass<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        for (i, group) in self.bind_groups().enumerate() {
            render_pass.set_bind_group(i as u32, group, &[]);
        }
    }
}
