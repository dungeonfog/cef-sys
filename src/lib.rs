#[cfg(target_os = "windows")]
mod bindings_windows;
#[cfg(target_os = "windows")]
pub use bindings_windows::*;

#[cfg(target_os = "linux")]
mod bindings_linux;
#[cfg(target_os = "linux")]
pub use bindings_linux::*;

#[cfg(target_os = "macos")]
mod bindings_macos;
#[cfg(target_os = "macos")]
pub use bindings_macos::*;
