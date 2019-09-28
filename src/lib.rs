#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(target_os = "windows")]
pub type HINSTANCE = std::os::windows::raw::HANDLE;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
