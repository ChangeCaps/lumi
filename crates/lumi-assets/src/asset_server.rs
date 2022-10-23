use std::{
    any::TypeId,
    path::{Path, PathBuf},
    sync::Arc,
};

use lumi_id::{Id, IdMap};
use lumi_task::TaskPool;
use lumi_util::{
    crossbeam::channel::{unbounded, Receiver, Sender},
    once_cell::sync::OnceCell,
    DashMap, HashMap,
};

use crate::{handle, Asset, AssetIo, AssetLoader, FileAssetIo, Handle, HandleId, LoadContext};

#[derive(Default)]
pub struct AssetServerBuilder {
    loaders: HashMap<TypeId, Box<dyn AssetLoader>>,
    io: Option<Arc<dyn AssetIo>>,
    task_pool: Option<TaskPool>,
}

impl AssetServerBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn add_loader<T: AssetLoader>(&mut self, loader: T) {
        self.loaders.insert(TypeId::of::<T>(), Box::new(loader));
    }

    #[inline]
    pub fn set_io(&mut self, io: Arc<dyn AssetIo>) {
        self.io = Some(io);
    }

    #[inline]
    pub fn set_task_pool(&mut self, task_pool: TaskPool) {
        self.task_pool = Some(task_pool);
    }

    #[inline]
    pub fn with_loader<T: AssetLoader>(mut self, loader: T) -> Self {
        self.add_loader(loader);
        self
    }

    #[inline]
    pub fn with_io(mut self, io: Arc<dyn AssetIo>) -> Self {
        self.io = Some(io);
        self
    }

    #[inline]
    pub fn with_task_pool(mut self, task_pool: TaskPool) -> Self {
        self.task_pool = Some(task_pool);
        self
    }

    #[inline]
    pub fn has_loader<T: AssetLoader>(&self) -> bool {
        self.loaders.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn has_io(&self) -> bool {
        self.io.is_some()
    }

    #[inline]
    pub fn has_task_pool(&self) -> bool {
        self.task_pool.is_some()
    }

    #[inline]
    pub fn build(self) -> AssetServer {
        let io = self.io.unwrap_or_else(|| Arc::new(FileAssetIo::new(".")));
        let task_pool = self.task_pool.unwrap_or_else(|| TaskPool::new().unwrap());

        AssetServer::new_internal(io, self.loaders, task_pool)
    }
}

struct Inner {
    tracker_tx: Sender<(TypeId, HandleId)>,
    tracker_rx: Receiver<(TypeId, HandleId)>,
    ext_to_loader: HashMap<String, Id<Box<dyn AssetLoader>>>,
    loaders: IdMap<Box<dyn AssetLoader>>,
    handles: DashMap<(TypeId, HandleId), Arc<dyn Asset>>,
    task_pool: TaskPool,
}

#[derive(Clone)]
pub struct AssetServer {
    inner: Arc<Inner>,
    io: Arc<dyn AssetIo>,
}

impl AssetServer {
    #[inline]
    pub fn builder() -> AssetServerBuilder {
        AssetServerBuilder::new()
    }

    fn new_internal(
        io: Arc<dyn AssetIo>,
        loaders_by_type: HashMap<TypeId, Box<dyn AssetLoader>>,
        task_pool: TaskPool,
    ) -> Self {
        let (tracker_tx, tracker_rx) = unbounded();

        let mut ext_to_loader = HashMap::default();
        let mut loaders = IdMap::new();

        for loader in loaders_by_type.into_values() {
            let id = Id::<_>::new();

            for ext in loader.extensions() {
                ext_to_loader.insert(ext.to_string(), id);
            }

            loaders.insert(id, loader);
        }

        Self {
            inner: Arc::new(Inner {
                tracker_tx,
                tracker_rx,
                ext_to_loader,
                loaders,
                handles: DashMap::default(),
                task_pool,
            }),
            io,
        }
    }

    pub fn io(&self) -> &Arc<dyn AssetIo> {
        &self.io
    }

    #[inline]
    fn get_loader(&self, ext: &str) -> Option<&Box<dyn AssetLoader>> {
        let id = self.inner.ext_to_loader.get(ext)?;
        self.inner.loaders.get(id)
    }

    #[inline]
    async fn load_inner<T: Asset>(self, handle: Handle<T>, path: PathBuf) {
        let ext = path.extension().unwrap().to_str().unwrap();
        let loader = self.get_loader(ext).unwrap();

        let bytes = self.io.read(&path).await.unwrap();

        let context = LoadContext {
            bytes: &bytes,
            handle: &handle,
            extension: ext,
            asset_server: &self,
        };

        loader.load(&context).await.unwrap();
    }

    #[inline]
    pub fn load<T: Asset>(&self, path: &Path) -> Handle<T> {
        let id = HandleId::from(path);
        let handle = (TypeId::of::<T>(), id);

        if let Some(inner) = self.inner.handles.get(&handle) {
            let inner: *const handle::Inner<T> =
                Arc::into_raw(inner.clone()) as *const handle::Inner<T>;
            let inner: Arc<handle::Inner<T>> = unsafe { Arc::from_raw(inner) };

            return Handle::from_inner(inner);
        }

        let handle = Handle {
            inner: Arc::new(handle::Inner {
                id,
                asset: OnceCell::<T>::new(),
                tracker: Some(self.inner.tracker_tx.clone()),
            }),
        };

        self.inner
            .task_pool
            .spawn(self.clone().load_inner(handle.clone(), path.to_path_buf()))
            .detach();

        handle
    }

    #[inline]
    pub fn clean(&self) {
        for handle in self.inner.tracker_rx.try_iter() {
            self.inner.handles.remove(&handle);
        }
    }
}
