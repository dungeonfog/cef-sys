fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");

    match target_os.as_ref().map(|x| &**x) {
        Ok("windows") => {
            println!("cargo:rustc-link-lib=cef_sandbox");
            println!("cargo:rustc-link-lib=libcef");

            // These two libraries are winapi libs, but they aren't available through winapi so we
            // link them here.
            println!("cargo:rustc-link-lib=wbemuuid");
            println!("cargo:rustc-link-lib=propsys");
        },
        Ok("linux") => {
            println!("cargo:rustc-link-lib=cef");
            println!("cargo:rustc-link-lib=EGL");
            println!("cargo:rustc-link-lib=GLESv2");
        }
        Ok("macos") => {
            println!("cargo:rustc-link-lib=cef_dll_wrapper");
        }
        _ => (),
    }
}
