use bindgen::builder;
fn main() {
    let mut bindings = builder()
        .header("S:/cef/git/chromium/src/everything.h")
        .whitelist_type("_?cef_.*")
        .whitelist_function("_?cef_.*")
        .whitelist_var("_?cef_.*")
        .blacklist_type("H[A-Z]+")
        .blacklist_type("MSG'")
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .bitfield_enum("cef_transition_type_t")
        .bitfield_enum("cef_v8_propertyattribute_t")
        .bitfield_enum("cef_v8_accesscontrol_t")
        .bitfield_enum("cef_drag_operations_mask_t")
        .bitfield_enum("cef_cert_status_t")
        .bitfield_enum("cef_file_dialog_mode_t")
        .bitfield_enum(".+flags.+");
    bindings.generate().unwrap().write_to_file("./bindings.rs").unwrap();
}
