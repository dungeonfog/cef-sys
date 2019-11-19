use bindgen::builder;
use std::env;
use std::iter;
use std::path::Path;
fn main() {
    let chromium_path = env::args()
        .skip(1)
        .next()
        .expect("Must pass chromium src path as first argument");
    let chromium_path = dunce::canonicalize(Path::new(&*chromium_path)).unwrap();
    let cef_path = chromium_path.join("cef");

    let include_paths = vec![
        cef_path.to_string_lossy().into_owned(),
        chromium_path.to_string_lossy().into_owned(),
    ];
    #[cfg(target_os = "windows")]
    let include = "INCLUDE";
    #[cfg(target_os = "windows")]
    let separator = ";";
    #[cfg(not(target_os = "windows"))]
    let include = "C_INCLUDE_PATH";
    #[cfg(not(target_os = "windows"))]
    let separator = ":";

    env::set_var(
        include,
        include_paths
            .into_iter()
            .chain(iter::once(env::var(include).unwrap_or(String::new())))
            .map(|s| s + separator)
            .collect::<String>(),
    );

    let bindings = builder()
        .header_contents("everything.h", include_str!("../everything.h"))
        .whitelist_type("_?cef_.*")
        .whitelist_function("_?cef_.*")
        .whitelist_var("_?cef_.*")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .bitfield_enum("cef_cert_status_t")
        .bitfield_enum("cef_v8_propertyattribute_t")
        .bitfield_enum("cef_urlrequest_flags_t")
        .bitfield_enum("cef_event_flags_t")
        .bitfield_enum("cef_context_menu_type_flags_t")
        .bitfield_enum("cef_context_menu_media_state_flags_t")
        .bitfield_enum("cef_context_menu_edit_state_flags_t")
        .bitfield_enum("cef_uri_unescape_rule_t")
        .bitfield_enum("cef_json_parser_options_t")
        .bitfield_enum("cef_json_writer_options_t")
        .bitfield_enum("cef_ssl_content_status_t")
        .bitfield_enum("cef_scheme_options_t")
        .blacklist_type("H[A-Z]+_*") // blacklist Windows handles
        .blacklist_type("(tag)?MSG")
        .generate()
        .unwrap();
    bindings.write_to_file("./bindings.rs").unwrap();
}
