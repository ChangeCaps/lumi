use std::{any::Any, borrow::Cow, collections::LinkedList};

use wgpu::{
    BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    ComputePass, PipelineLayout, RenderPass, ShaderStages,
};

use crate::{
    bind::{Bind, BindingLayoutEntry, SharedBindingResource},
    shader::Shader,
    util::HashMap,
    Device, Queue, SharedBindGroup, SharedDevice,
};

#[derive(Debug)]
pub struct BindingLocation {
    pub binding: u32,
    pub group: u32,
}

#[derive(Debug)]
pub struct BindingsLayout {
    visibility: Option<ShaderStages>,
    entries: LinkedList<BindingLayoutEntry>,
    bindings: HashMap<Cow<'static, str>, BindingLocation>,
}

impl BindingsLayout {
    pub fn new() -> Self {
        Self {
            visibility: None,
            entries: LinkedList::new(),
            bindings: HashMap::default(),
        }
    }

    pub fn set_visibility(&mut self, visibility: ShaderStages) {
        self.visibility = Some(visibility);
    }

    pub fn with_visibility(mut self, visibility: ShaderStages) -> Self {
        self.set_visibility(visibility);
        self
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

    pub fn create_bind_group_layouts(&self, device: &Device) -> Vec<BindGroupLayout> {
        let mut entries: Vec<Vec<BindGroupLayoutEntry>> = Vec::new();

        for entry in self.entries.iter() {
            let binding = self.bindings.get(&entry.name).unwrap();

            while entries.len() <= binding.group as usize {
                entries.push(Vec::new());
            }

            let entries = &mut entries[binding.group as usize];

            let entry = BindGroupLayoutEntry {
                binding: binding.binding,
                visibility: self.visibility.unwrap_or(entry.visibility),
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

    pub fn create_pipeline_layout(&self, device: &Device) -> PipelineLayout {
        let bind_group_layouts = self.create_bind_group_layouts(device);
        let bind_group_layouts = bind_group_layouts.iter().collect::<Vec<_>>();

        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        })
    }

    pub fn create_bindings(&self, device: &Device) -> Bindings {
        let mut groups: Vec<BindingGroup> = self
            .create_bind_group_layouts(device)
            .into_iter()
            .map(|layout| BindingGroup {
                entries: Vec::new(),
                layout,
                bind_group: None,
            })
            .collect();

        for entry in self.entries.iter() {
            let binding = self.bindings.get(&entry.name).unwrap();

            let group_entry = BindingGroupEntry {
                name: entry.name.clone(),
                binding: binding.binding,
                resource: None,
                state: None,
            };

            let group = &mut groups[binding.group as usize];
            group.entries.push(group_entry);
        }

        Bindings { groups }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BindingKind {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

struct BindingGroupEntry {
    name: Cow<'static, str>,
    binding: u32,
    resource: Option<SharedBindingResource>,
    state: Option<Box<dyn Any + Send + Sync>>,
}

impl BindingGroupEntry {
    fn update(&mut self, resource: SharedBindingResource) -> bool {
        if self.resource.as_ref() != Some(&resource) {
            self.resource = Some(resource);
            true
        } else {
            false
        }
    }
}

struct BindingGroup {
    entries: Vec<BindingGroupEntry>,
    layout: BindGroupLayout,
    bind_group: Option<SharedBindGroup>,
}

type EntryIndex = (usize, usize);

pub struct Bindings {
    groups: Vec<BindingGroup>,
}

impl Bindings {
    #[inline]
    pub fn bind<T: Bind>(&mut self, device: &Device, queue: &Queue, bind: &T) {
        bind.bind(device, queue, self);
    }

    #[inline]
    pub fn get_index(&self, name: &str) -> Option<EntryIndex> {
        for (group_index, group) in self.groups.iter().enumerate() {
            for (entry_index, entry) in group.entries.iter().enumerate() {
                if entry.name == name {
                    return Some((group_index, entry_index));
                }
            }
        }

        None
    }

    #[inline]
    pub unsafe fn get_state<T: Any + Default + Send + Sync>(
        &mut self,
        (group, entry): EntryIndex,
    ) -> &mut T {
        let entry = unsafe {
            self.groups
                .get_unchecked_mut(group)
                .entries
                .get_unchecked_mut(entry)
        };

        let state = entry.state.get_or_insert_with(|| Box::new(T::default()));
        state.downcast_mut::<T>().unwrap()
    }

    #[inline]
    pub unsafe fn update_resource(
        &mut self,
        (group, entry): EntryIndex,
        resource: SharedBindingResource,
    ) {
        let group = unsafe { self.groups.get_unchecked_mut(group) };
        let entry = unsafe { group.entries.get_unchecked_mut(entry) };
        if entry.update(resource) {
            group.bind_group = None;
        }
    }

    #[inline]
    pub fn update_bind_groups(&mut self, device: &Device) {
        for group in self.groups.iter_mut() {
            if group.bind_group.is_none() {
                let entries = group
                    .entries
                    .iter()
                    .map(|entry| {
                        let resource = entry.resource.as_ref().unwrap();
                        wgpu::BindGroupEntry {
                            binding: entry.binding,
                            resource: resource.as_binding_resource(),
                        }
                    })
                    .collect::<Vec<_>>();

                let bind_group = device.create_shared_bind_group(&BindGroupDescriptor {
                    label: None,
                    layout: &group.layout,
                    entries: &entries,
                });

                group.bind_group = Some(bind_group);
            }
        }
    }

    #[inline]
    pub fn bind_groups(&self) -> impl Iterator<Item = &SharedBindGroup> {
        self.groups
            .iter()
            .filter_map(|group| group.bind_group.as_ref())
    }

    #[inline]
    pub fn apply<'a>(&'a self, pass: &mut RenderPass<'a>) {
        for (i, group) in self.bind_groups().enumerate() {
            pass.set_bind_group(i as u32, group, &[]);
        }
    }

    #[inline]
    pub fn apply_compute<'a>(&'a self, pass: &mut ComputePass<'a>) {
        for (i, group) in self.bind_groups().enumerate() {
            pass.set_bind_group(i as u32, group, &[]);
        }
    }
}
