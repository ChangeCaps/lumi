use std::{
    borrow::Cow,
    collections::{HashMap, LinkedList},
};

use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry};

use crate::{Bind, BindingLayoutEntry, SharedBindingResource, SharedDevice};

pub struct BindingsBuilder {
    entries: LinkedList<BindingLayoutEntry>,
    bindings: HashMap<Cow<'static, str>, u32>,
}

impl BindingsBuilder {
    pub fn new() -> Self {
        Self {
            entries: LinkedList::new(),
            bindings: HashMap::new(),
        }
    }

    pub fn from_shader(module: &naga::Module) -> Self {
        let mut this = Self::new();

        for (_, variable) in module.global_variables.iter() {
            if let Some(ref name) = variable.name {
                if let Some(ref binding) = variable.binding {
                    this.add_binding(name.clone(), binding.binding);
                }
            }
        }

        this
    }

    pub fn add_binding(&mut self, name: impl Into<Cow<'static, str>>, binding: u32) {
        self.bindings.insert(name.into(), binding);
    }

    pub fn with_binding(mut self, name: impl Into<Cow<'static, str>>, binding: u32) -> Self {
        self.add_binding(name, binding);
        self
    }

    pub fn with_bindings<T, I>(mut self, bindings: I) -> Self
    where
        T: Into<Cow<'static, str>>,
        I: Iterator<Item = (T, u32)>,
    {
        let bindings = bindings.map(|(name, binding)| (name.into(), binding));
        self.bindings.extend(bindings);
        self
    }

    pub fn bind<T: Bind>(mut self) -> Self {
        self.entries.append(&mut T::entries());
        self
    }

    pub fn push(mut self, entry: BindingLayoutEntry) -> Self {
        self.entries.push_back(entry);
        self
    }

    pub fn append(mut self, mut entries: LinkedList<BindingLayoutEntry>) -> Self {
        self.entries.append(&mut entries);
        self
    }

    pub fn build(self, device: &SharedDevice) -> Bindings {
        #[derive(Default)]
        struct Group {
            bindings: HashMap<Cow<'static, str>, BindingGroupEntry>,
            entries: Vec<BindGroupLayoutEntry>,
        }

        let mut groups: Vec<Group> = Vec::new();

        for entry in self.entries {
            while groups.len() <= entry.group as usize {
                groups.push(Group::default());
            }

            let group = &mut groups[entry.group as usize];

            let binding = self
                .bindings
                .get(&entry.name)
                .copied()
                .unwrap_or_else(|| group.bindings.len() as u32);

            group
                .bindings
                .insert(entry.name, BindingGroupEntry::new(binding));

            let entry = BindGroupLayoutEntry {
                binding: entry.group,
                visibility: entry.visibility,
                ty: entry.ty,
                count: entry.count,
            };

            group.entries.push(entry);
        }

        let groups = groups
            .into_iter()
            .map(|group| {
                let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &group.entries,
                });

                BindingGroup {
                    bindings: group.bindings,
                    layout,
                    bind_group: None,
                }
            })
            .collect();

        Bindings { groups }
    }
}

struct BindingGroupEntry {
    resource: Option<SharedBindingResource>,
    binding: u32,
}

impl BindingGroupEntry {
    pub const fn new(binding: u32) -> Self {
        Self {
            resource: None,
            binding,
        }
    }
}

struct BindingGroup {
    bindings: HashMap<Cow<'static, str>, BindingGroupEntry>,
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
                .unwrap_or_else(|| panic!("Binding '{}' not bound", name));

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
    pub fn build() -> BindingsBuilder {
        BindingsBuilder::new()
    }

    pub fn layouts(&self) -> Vec<&BindGroupLayout> {
        self.groups.iter().map(|g| &g.layout).collect()
    }

    #[track_caller]
    pub fn bind(&mut self, device: &SharedDevice, bind: &impl Bind) {
        for binding in bind.bindings(device) {
            let group = &mut self.groups[binding.group as usize];

            if let Some(entry) = group.bindings.get_mut(&binding.name) {
                let resource = Some(binding.resource);

                if entry.resource != resource {
                    entry.resource = resource;

                    // invalidate bind group
                    group.bind_group = None;
                }
            } else {
                panic!("Binding '{}' doesn't exist in layout", binding.name);
            }
        }
    }

    pub fn update_bind_group(&mut self, device: &SharedDevice) {
        for group in self.groups.iter_mut() {
            if group.bind_group.is_none() {
                group.bind_group = Some(group.create_bind_group(device));
            }
        }
    }

    #[track_caller]
    pub fn bind_groups(&self) -> Vec<&BindGroup> {
        self.groups
            .iter()
            .map(|g| g.bind_group.as_ref().expect("BindGroup not created"))
            .collect()
    }
}
