use bindgen::builder;
use std::env;
use std::iter;
use std::path::{Path, PathBuf};
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
    #[cfg(not(target_os = "windows"))]
    let include = "C_INCLUDE_PATH";

    env::set_var(
        include,
        include_paths
            .into_iter()
            .chain(iter::once(env::var(include).unwrap_or(String::new())))
            .map(|s| s + ";")
            .collect::<String>(),
    );

    let bindings = builder()
        .header_contents("everything.h", include_str!("../everything.h"))
        .whitelist_type("_?cef_.*")
        .whitelist_function("_?cef_.*")
        .whitelist_var("_?cef_.*")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .bitfield_enum("cef_transition_type_t")
        .bitfield_enum("cef_v8_propertyattribute_t")
        .bitfield_enum("cef_v8_accesscontrol_t")
        .bitfield_enum("cef_drag_operations_mask_t")
        .bitfield_enum("cef_cert_status_t")
        .bitfield_enum("cef_file_dialog_mode_t")
        .bitfield_enum(".+flags.+")
        .generate()
        .unwrap();
    bindings.write_to_file("./bindings.rs").unwrap();
}
