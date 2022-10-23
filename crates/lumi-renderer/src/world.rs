use lumi_id::IdSet;
use lumi_util::crossbeam::channel::Receiver;
use lumi_world::{World, WorldChange, WorldId};

pub struct PreparedWorld {
    pub id: WorldId,
    pub changes: Receiver<WorldChange>,
    pub objects: IdSet,
}

#[derive(Default)]
pub struct PreparedWorlds {
    worlds: Vec<PreparedWorld>,
}

#[allow(dead_code)]
impl PreparedWorlds {
    #[inline]
    pub fn contains(&self, id: WorldId) -> bool {
        self.worlds.iter().any(|w| w.id == id)
    }

    #[inline]
    pub fn get_index(&self, id: WorldId) -> Option<usize> {
        self.worlds.iter().position(|w| w.id == id)
    }

    #[inline]
    pub fn get(&self, id: WorldId) -> Option<&PreparedWorld> {
        self.worlds.iter().find(|w| w.id == id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: WorldId) -> Option<&mut PreparedWorld> {
        self.worlds.iter_mut().find(|w| w.id == id)
    }

    #[inline]
    pub fn subscribe(&mut self, world: &World) -> &mut PreparedWorld {
        if let Some(index) = self.get_index(world.id()) {
            return &mut self.worlds[index];
        }

        let world = PreparedWorld {
            id: world.id(),
            changes: world.subscribe_changes(),
            objects: IdSet::new(),
        };

        self.worlds.push(world);
        self.worlds.last_mut().unwrap()
    }
}
