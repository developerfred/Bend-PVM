pub mod core;

use self::core::StdlibCore;

/// Initialize the standard library
pub fn init_stdlib() -> StdlibCore {
    StdlibCore::new()
}

/// Get the standard library core
pub fn get_stdlib() -> StdlibCore {
    StdlibCore::new()
}