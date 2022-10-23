mod asset_loader;
mod asset_server;
mod handle;
mod io;

pub use asset_loader::*;
pub use asset_server::*;
pub use handle::*;
pub use io::*;

pub trait Asset: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Asset for T {}
