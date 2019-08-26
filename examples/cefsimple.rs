use cef_sys::*;
use std::sync::Mutex;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::{ptr::hash, hash::Hasher, os::raw::{c_int, c_uint, c_void}, ffi::CString, mem::size_of};
use lazy_static::lazy_static;
#[cfg(windows)]
use winapi::um::{libloaderapi::GetModuleHandleA, winuser::{
    WS_OVERLAPPEDWINDOW,
    WS_CLIPCHILDREN,
    WS_CLIPSIBLINGS,
    WS_VISIBLE,
    CW_USEDEFAULT,
}};
use libc::free;

lazy_static! {
    static ref REFCOUNT: Mutex<HashMap<u64, usize>> = Mutex::new(HashMap::new());
}

struct CefBaseRefCounted;

impl CefBaseRefCounted {
    fn hash(ref_counted: *const cef_base_ref_counted_t) -> u64 {
        let mut hasher = DefaultHasher::new();
        hash(ref_counted, &mut hasher);
        hasher.finish()
    }
    pub fn new(size: usize) -> cef_base_ref_counted_t {
        let instance = cef_base_ref_counted_t {
            size,
            add_ref: Some(CefBaseRefCounted::add_ref),
            release: Some(CefBaseRefCounted::release),
            has_one_ref: Some(CefBaseRefCounted::has_one_ref),
            has_at_least_one_ref: Some(CefBaseRefCounted::has_at_least_one_ref),
        };
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            ref_count.insert(Self::hash(&instance), 1);
        }
        instance
    }

    extern "C" fn add_ref(ref_counted: *mut cef_base_ref_counted_t) {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            if let Some(c) = ref_count.get_mut(&Self::hash(ref_counted)) {
                *c += 1;
            }
        }
    }
    extern "C" fn release(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            let hash = Self::hash(ref_counted);
            if let Some(c) = ref_count.get_mut(&hash) {
                *c -= 1;
                if *c == 0 {
                    ref_count.remove(&hash);
                    unsafe { free(ref_counted as *mut c_void); }
                    return 1;
                }
            }
        }
        return 0;
    }
    extern "C" fn has_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            let hash = Self::hash(ref_counted);
            if let Some(c) = ref_count.get_mut(&hash) {
                if *c == 1 {
                    return 1;
                }
            }
        }
        return 0;
    }
    extern "C" fn has_at_least_one_ref(ref_counted: *mut cef_base_ref_counted_t) -> c_int {
        if let Ok(ref mut ref_count) = REFCOUNT.lock() {
            let hash = Self::hash(ref_counted);
            if ref_count.contains_key(&hash) {
                return 1;
            }
        }
        return 0;
    }
}

#[repr(C)]
struct CefApp(cef_app_t);

impl CefApp {
    pub fn new() -> Self {
        CefApp(cef_app_t {
            base: CefBaseRefCounted::new(size_of::<cef_app_t>()),
            on_before_command_line_processing: Some(Self::on_before_command_line_processing),
            on_register_custom_schemes: Some(Self::on_register_custom_schemes),
            get_resource_bundle_handler: Some(Self::get_resource_bundle_handler),
            get_browser_process_handler: Some(Self::get_browser_process_handler),
            get_render_process_handler: Some(Self::get_render_process_handler),
        })
    }

    extern "C" fn on_before_command_line_processing(_self: *mut cef_app_t, _process_type: *const cef_string_t, _command_line: *mut cef_command_line_t) {
    }
    extern "C" fn on_register_custom_schemes(_self: *mut cef_app_t, _registrar: *mut cef_scheme_registrar_t) {
    }
    extern "C" fn get_resource_bundle_handler(_self: *mut cef_app_t) -> *mut cef_resource_bundle_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_browser_process_handler(_self: *mut cef_app_t) -> *mut cef_browser_process_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_render_process_handler(_self: *mut cef_app_t) -> *mut cef_render_process_handler_t {
        std::ptr::null_mut()
    }
}

struct CefSettings;

impl CefSettings {
    pub fn new(log_severity: cef_log_severity_t, no_sandbox: bool, locales_path: &str) -> cef_settings_t {
        cef_settings_t {
            size: size_of::<cef_settings_t>(),
            no_sandbox: if no_sandbox { 1 } else { 0 },
            log_severity,
            locales_dir_path: CefString::new(locales_path).into(),
            ..Default::default()
        }
    }
}

struct CefWindowInfo;

impl CefWindowInfo {
    #[cfg(windows)]
    pub fn new(style: DWORD, x: c_int, y: c_int, width: c_int, height: c_int, name: &str) -> cef_window_info_t {
        cef_window_info_t {
            style,
            x,
            y,
            width,
            height,
            window_name: CefString::new(name).into(),
            ..Default::default()
        }
    }
    #[cfg(not(windows))]
    pub fn new(x: c_uint, y: c_uint, width: c_uint, height: c_uint, name: &str) -> cef_window_info_t {
        cef_window_info_t {
            x,
            y,
            width,
            height,
            window_name: CefString::new(name).into(),
            ..Default::default()
        }
    }
}

struct CefString<'a>(&'a str);

impl<'a> CefString<'a> {
    pub fn new(source: &'a str) -> Self {
        Self(source)
    }
}

impl<'a> std::convert::Into<cef_string_t> for CefString<'a> {
    fn into(self) -> cef_string_t {
        let mut result = cef_string_t::default();
        let len = self.0.len();
        unsafe {
            cef_string_utf8_to_utf16(self.0.as_ptr() as *const std::os::raw::c_char, len, &mut result);
        }
        result
    }
}

struct CefBrowserSettings;

impl CefBrowserSettings {
    pub fn new() -> cef_browser_settings_t {
        cef_browser_settings_t {
            size: size_of::<cef_browser_settings_t>(),
            ..Default::default()
        }
    }
}

#[repr(C)]
struct CefClient(cef_client_t);

impl CefClient {
    pub fn new() -> Self {
        CefClient(cef_client_t {
            base: CefBaseRefCounted::new(size_of::<cef_client_t>()),
            get_audio_handler: Some(Self::get_audio_handler),
            get_context_menu_handler: Some(Self::get_context_menu_handler),
            get_dialog_handler: Some(Self::get_dialog_handler),
            get_display_handler: Some(Self::get_display_handler),
            get_download_handler: Some(Self::get_download_handler),
            get_drag_handler: Some(Self::get_drag_handler),
            get_find_handler: Some(Self::get_find_handler),
            get_focus_handler: Some(Self::get_focus_handler),
            get_jsdialog_handler: Some(Self::get_jsdialog_handler),
            get_keyboard_handler: Some(Self::get_keyboard_handler),
            get_life_span_handler: Some(Self::get_life_span_handler),
            get_load_handler: Some(Self::get_load_handler),
            get_render_handler: Some(Self::get_render_handler),
            get_request_handler: Some(Self::get_request_handler),
            on_process_message_received: Some(Self::on_process_message_received),
        })
    }

    extern "C" fn get_audio_handler(_self: *mut cef_client_t) -> *mut cef_audio_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_context_menu_handler(_self: *mut cef_client_t) -> *mut cef_context_menu_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_dialog_handler(_self: *mut cef_client_t) -> *mut cef_dialog_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_display_handler(_self: *mut cef_client_t) -> *mut cef_display_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_download_handler(_self: *mut cef_client_t) -> *mut cef_download_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_drag_handler(_self: *mut cef_client_t) -> *mut cef_drag_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_find_handler(_self: *mut cef_client_t) -> *mut cef_find_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_focus_handler(_self: *mut cef_client_t) -> *mut cef_focus_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_jsdialog_handler(_self: *mut cef_client_t) -> *mut cef_jsdialog_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_keyboard_handler(_self: *mut cef_client_t) -> *mut cef_keyboard_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_life_span_handler(_self: *mut cef_client_t) -> *mut cef_life_span_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_load_handler(_self: *mut cef_client_t) -> *mut cef_load_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_render_handler(_self: *mut cef_client_t) -> *mut cef_render_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn get_request_handler(_self: *mut cef_client_t) -> *mut cef_request_handler_t {
        std::ptr::null_mut()
    }
    extern "C" fn on_process_message_received(
            _self: *mut cef_client_t,
            _browser: *mut cef_browser_t,
            _frame: *mut cef_frame_t,
            _source_process: cef_process_id_t,
            _message: *mut cef_process_message_t,
        ) -> ::std::os::raw::c_int {
        0
    }
}

fn main() {
    let mut app = CefApp::new();
    #[cfg(not(windows))]
    let main_args = cef_main_args_t {
        argc: 1,
        argv: [CString::new("cefsimple").unwrap().into_raw()].as_mut_ptr(),
    };
    unsafe {
        cef_enable_highdpi_support();
        #[cfg(windows)]
        let main_args = cef_main_args_t {
            instance: GetModuleHandleA(std::ptr::null()) as HINSTANCE
        };

        let result = cef_execute_process(&main_args, &mut app.0, std::ptr::null_mut());
        if result >= 0 {
            std::process::exit(result);
        }

        let settings = CefSettings::new(cef_log_severity_t::LOGSEVERITY_VERBOSE, true, "./locales");
        cef_initialize(&main_args, &settings, &mut app.0, std::ptr::null_mut());

        #[cfg(windows)]
        let window_info = CefWindowInfo::new(WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, "cefcapi Rust example");
        #[cfg(not(windows))]
        let window_info = CefWindowInfo::new(0, 0, 1920, 1080, "cefcapi Rust example");

        let url = CefString::new("https://www.youtube.com").into();
        let browser_settings = CefBrowserSettings::new();


        let mut client = CefClient::new();

        println!("cef_browser_host_create_browser");
        cef_browser_host_create_browser(&window_info, &mut client.0, &url, &browser_settings, std::ptr::null_mut(), std::ptr::null_mut());

        println!("cef_run_message_loop");
        cef_run_message_loop();

        println!("cef_shutdown");
        cef_shutdown();
    }
}