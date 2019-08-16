use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=cef_sandbox");
    println!("cargo:rustc-link-lib=libcef");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.hpp")
        .clang_arg("-I../cef_binary_76.1.9+g2cf916e+chromium-76.0.3809.87_windows64")
        .whitelist_type("cef_life_span_handler_t")
        .whitelist_type("cef_app_t")
        .whitelist_type("cef_command_line_t")
        .whitelist_type("cef_resource_bundle_handler_t")
        .whitelist_type("cef_scheme_registrar_t")
        .whitelist_type("cef_resource_bundle_handler_t")
        .whitelist_type("cef_browser_process_handler_t")
        .whitelist_type("cef_render_process_handler_t")
        .whitelist_type("cef_main_args_t")
        .whitelist_function("cef_execute_process")
        .whitelist_type("cef_settings_t")
        .whitelist_function("cef_initialize")
        .whitelist_type("cef_window_t")
        .whitelist_type("cef_string_t")
        .whitelist_function("cef_string_utf8_to_utf16")
        .whitelist_type("cef_browser_settings_t")
        .whitelist_type("cef_browser_host_t")
        .whitelist_type("cef_client_t")
        .whitelist_type("cef_audio_handler_t")
        .whitelist_type("cef_context_menu_handler_t")
        .whitelist_type("cef_dialog_handler_t")
        .whitelist_type("cef_display_handler_t")
        .whitelist_type("cef_download_handler_t")
        .whitelist_type("cef_drag_handler_t")
        .whitelist_type("cef_find_handler_t")
        .whitelist_type("cef_focus_handler_t")
        .whitelist_type("cef_jsdialog_handler_t")
        .whitelist_type("cef_keyboard_handler_t")
        .whitelist_type("cef_life_span_handler_t")
        .whitelist_type("cef_load_handler_t")
        .whitelist_type("cef_render_handler_t")
        .whitelist_type("cef_screen_info_t")
        .whitelist_type("cef_request_handler_t")
        .whitelist_type("cef_browser_t")
        .whitelist_type("cef_frame_t")
        .whitelist_type("cef_process_message_t")
        .whitelist_function("cef_browser_host_create_browser")
        .whitelist_function("cef_browser_host_create_browser_sync")
        .whitelist_function("cef_run_message_loop")
        .whitelist_function("cef_quit_message_loop")
        .whitelist_function("cef_do_message_loop_work")
        .whitelist_function("cef_shutdown")
        .whitelist_function("cef_enable_highdpi_support")
        .derive_copy(false)
        .derive_default(true)
        .generate_comments(true)
        .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
        .rustfmt_bindings(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}