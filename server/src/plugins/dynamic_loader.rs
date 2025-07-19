//! Dynamic plugin loader stub for hot-reloading

use std::path::Path;
use libloading::{Library, Symbol};

/// Loads a plugin from a dynamic library (.so/.dll/.dylib)
pub fn load_plugin<P: AsRef<Path>>(path: P) -> Result<Library, libloading::Error> {
    unsafe { Library::new(path) }
}

// TODO: Define trait object symbol conventions and safe dynamic loading for plugins
