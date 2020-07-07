use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write, Seek, SeekFrom};

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_os = target_os.as_ref().map(|x| &**x);

    let lib_dir_env_var = "CARGO_CEF_SYS_LIB_OUT_DIR";
    let archive_dir_env_var = "CARGO_CEF_SYS_ARCHIVE_OUT_DIR";
    let unpack_sentinel_env_var = "CARGO_CEF_SYS_UNPACK_SENTINEL";
    println!("cargo:rerun-if-env-changed={}", lib_dir_env_var);
    println!("cargo:rerun-if-env-changed={}", archive_dir_env_var);
    println!("cargo:rerun-if-env-changed={}", unpack_sentinel_env_var);
    let lib_dir_env = std::env::var(lib_dir_env_var).ok();
    let archive_dir_env_var = std::env::var(archive_dir_env_var).ok();
    let unpack_sentinel_env_var = std::env::var(unpack_sentinel_env_var).ok();


    let cef_version = "83.4.2+gc8d4f85+chromium-83.0.4103.106";
    let cef_platform = match std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => cef_installer::Platform::Windows,
        "linux" => cef_installer::Platform::Linux,
        "macos" => cef_installer::Platform::MacOS,
        p => panic!("platform {} unsupported", p),
    };
    let opt_level = cef_installer::OptLevel::Release;
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let mut unpack_cef = true;

    let unpack_sentinel_path = match unpack_sentinel_env_var {
        Some(path) => PathBuf::from(path),
        None => out_dir.join("unpack_sentinel"),
    };

    let targz_dir = match archive_dir_env_var {
        Some(dir) => {
            std::fs::create_dir_all(&dir).expect("could not create targz dir");
            dunce::canonicalize(PathBuf::from(&dir)).expect("could not canonicalize archive dir")
        },
        None => out_dir.clone()
    };
    let lib_dir = match lib_dir_env {
        Some(dir) => {
            std::fs::create_dir_all(&dir).expect("could not create lib dir");
            dunce::canonicalize(PathBuf::from(&dir)).expect("could not canonicalize lib dir")
        },
        None => out_dir.clone(),
    };
    let unpack_sentinel_file_contents = format!(
        "{};{};{}",
        cef_version,
        targz_dir.display(),
        lib_dir.display(),
    );
    if let Ok(actual_contents) = fs::read_to_string(&unpack_sentinel_path) {
        if unpack_sentinel_file_contents == actual_contents {
            unpack_cef = false;
        }
    }
    let libcef_dll_project_dir = out_dir.join("libcef_dll");
    let header_dir = libcef_dll_project_dir.join("include");
    let libcef_dll_src_dir = libcef_dll_project_dir.join("libcef_dll");
    let cmake_macros_dir = libcef_dll_project_dir.join("cmake");

    // we ignore the unpack sentinel on macos because it needs to unpack the archive to build
    // ibcef_dll_wrapper
    if unpack_cef && target_os != Ok("macos") {
        cef_installer::download_cef(
            cef_version,
            cef_platform,
            opt_level,
            Some(&targz_dir),
            Some(&lib_dir),
            Some(&header_dir),
            Some(&libcef_dll_src_dir),
            Some(&cmake_macros_dir),
            false,
        ).unwrap();

        fs::write(&unpack_sentinel_path, &unpack_sentinel_file_contents).ok();
    }

    println!("cargo:rustc-link-search={}", lib_dir.display());

    match target_os {
        Ok("windows") => {
            #[cfg(feature = "sandbox")]
            {
                println!("cargo:rustc-link-lib=cef_sandbox");
            }
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
            remove_find_package_dep(&cmake_macros_dir.join("cef_macros.cmake"));
            remove_find_package_dep(&cmake_macros_dir.join("cef_variables.cmake"));

            let cmake_file = format!("
                cmake_minimum_required(VERSION 3.0)
                project(dll_wrapper)
                set(CMAKE_MODULE_PATH ${{CMAKE_MODULE_PATH}} \"{}\")
                include(\"cef_macros\")
                include(\"cef_variables\")
                include_directories(\"{}\")
                add_subdirectory(\"{}\")
                install(TARGETS libcef_dll_wrapper DESTINATION .)
            ",
                cmake_macros_dir.display().to_string().replace("\\", "/"),
                libcef_dll_project_dir.display().to_string().replace("\\", "/"),
                libcef_dll_src_dir.display().to_string().replace("\\", "/"),
            );
            fs::write(&libcef_dll_project_dir.join("CMakeLists.txt"), &cmake_file).unwrap();

            let dst = cmake::Config::new(&libcef_dll_project_dir).generator("Ninja").build();
            println!("cargo:rustc-link-search=native={}", dst.display());
            println!("cargo:rustc-link-lib=static=cef_dll_wrapper");

            let framework_dir = lib_dir.join("Chromium Embedded Framework.framework");
            assert!(framework_dir.exists());
            println!("cargo:rustc-env=CEF_SYS_FRAMEWORK_PATH={}", framework_dir.display());
        }
        _ => (),
    }
}

fn remove_find_package_dep(path: &Path) {
    let mut cmake_macros_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("cef_macros.cmake not found in expected location");
    let mut cmake_macros_str = String::new();
    cmake_macros_file.read_to_string(&mut cmake_macros_str).unwrap();

    #[cfg(target_os = "windows")]
    { cmake_macros_str = cmake_macros_str.replace("\r\n", "\n"); }

    let new_str = cmake_macros_str.replace(CEF_MACROS_REMOVE, "");
    cmake_macros_str = new_str;

    #[cfg(target_os = "windows")]
    { cmake_macros_str = cmake_macros_str.replace("\n", "\r\n"); }

    cmake_macros_file.seek(SeekFrom::Start(0)).unwrap();
    cmake_macros_file.set_len(cmake_macros_str.len() as u64).unwrap();
    cmake_macros_file.write(cmake_macros_str.as_bytes()).unwrap();
}

const CEF_MACROS_REMOVE: &str = "if(NOT DEFINED _CEF_ROOT_EXPLICIT)
  message(FATAL_ERROR \"Use find_package(CEF) to load this file.\")
endif()";
