#[cfg(windows)]
mod bindings_windows;
#[cfg(windows)]
pub use bindings_windows::*;

#[cfg(linux)]
mod bindings_linux;
#[cfg(linux)]
pub use bindings_linux::*;
