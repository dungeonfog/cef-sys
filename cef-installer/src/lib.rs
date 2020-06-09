use regex::Regex;
use urlencoding::encode as urlencode;
use std::{path::Path, fs, io::{Cursor, Read, BufReader}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OptLevel {
    Debug,
    Release
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl Platform {
    fn str(self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::Linux => "linux",
            Platform::MacOS => "macosx",
        }
    }
}

pub fn download_cef(
    version: &str,
    platform: Platform,
    opt_level: OptLevel,
    targz_dir: Option<&Path>,
    lib_dir: Option<&Path>,
    header_dir: Option<&Path>,
    libcef_dll_src_dir: Option<&Path>,
    cmake_macros_dir: Option<&Path>,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = format!("cef_binary_{}_{}64.tar.bz2", &version, &platform.str());

    let reader: Box<dyn Read> = {
        let targz_path = targz_dir
            .map(|path| path.join(&file_name));
        let targz_file = targz_path.as_ref()
            .and_then(|path| fs::File::open(path).ok());

        match targz_file {
            Some(file) => Box::new(BufReader::new(file)),
            None => {
                let url = format!(
                    "http://opensource.spotify.com/cefbuilds/{}",
                    urlencode(&file_name),
                );

                if !quiet {
                    eprintln!("Fetching from {}", url);
                }

                let response = ureq::get(&url).call();
                if response.status() != 200 {
                    panic!("Server responded with HTTP result {}", response.status());
                }
                let mut buf = Vec::new();
                response.into_reader().read_to_end(&mut buf)?;

                if let Some(targz_dir) = targz_dir {
                    fs::create_dir_all(targz_dir).ok();
                }
                if let Some(targz_path) = targz_path {
                    fs::write(targz_path, &buf).ok();
                }

                Box::new(Cursor::new(buf))
            }
        }
    };
    let tar_file = bzip2::read::BzDecoder::new(reader);
    let mut archive = tar::Archive::new(tar_file);

    let mut mappings: Vec<(Regex, String)> = Vec::new();
    if let Some(lib_dir) = lib_dir {
        let maps = match platform {
            Platform::Windows => vec![
                (format!(r"^[^/]+/{:?}/([^/]+\.(lib|dll|bin))$", opt_level), "${1}"),
                (format!(r"^[^/]+/{:?}/(swiftshader/[^/]+\.dll)$", opt_level), "${1}"),
                (r"^[^/]+/Resources/icudtl\.dat$".to_owned(), "icudtl.dat"),
                (r"^[^/]+/Resources/((locales/)?[^/]+\.pak)$".to_owned(), "${1}"),
            ],
            Platform::Linux => vec![
                (format!(r"^[^/]+/{:?}/([^/]+\.(so|bin))$", opt_level), "${1}"),
                (format!(r"^[^/]+/{:?}/(swiftshader/[^/]+\.so)$", opt_level), "${1}"),
                (r"^[^/]+/Resources/icudtl\.dat$".to_owned(), "icudtl.dat"),
                (r"^[^/]+/Resources/((locales/)?[^/]+\.pak)$".to_owned(), "${1}"),
            ],
            Platform::MacOS => vec![(
                format!(r"^[^/]+/{:?}/(Chromium Embedded Framework\.framework/.+)$", opt_level),
                "${1}",
            )],
        };
        mappings.extend(maps.into_iter().map(|(src, dest)| (Regex::new(&src).unwrap(), format!("{}/{}", lib_dir.display(), dest))));
    }

    if let Some(header_dir) = header_dir {
        mappings.push((
            Regex::new(r#"^[^/]+/include/(.+\.h)"#).unwrap(),
            format!("{}/{}", header_dir.display(), "${1}")
        ));
    }

    if let Some(libcef_dll_src_dir) = libcef_dll_src_dir {
        mappings.push((
            Regex::new(r#"^[^/]+/libcef_dll/(.+)"#).unwrap(),
            format!("{}/{}", libcef_dll_src_dir.display(), "${1}")
        ));
    }

    if let Some(cmake_macros_dir) = cmake_macros_dir {
        mappings.push((
            Regex::new(r#"^[^/]+/cmake/(.+)"#).unwrap(),
            format!("{}/{}", cmake_macros_dir.display(), "${1}")
        ));
    }

    for entry in archive.entries()? {
        if let Ok(mut entry) = entry {
            if let Ok(path) = entry.path() {
                let path_string = path.to_string_lossy().to_string();
                for (regex, destination) in mappings.iter() {
                    if regex.is_match(&path_string) {
                        let filename = regex
                            .replace(&path_string, destination.as_str())
                            .to_string();
                        let path = std::path::Path::new(&filename);
                        if let Some(folder) = path.parent() {
                            let _ = std::fs::create_dir_all(folder);
                        }
                        if !path.exists() {
                            if !quiet {
                                eprintln!("Writing {}...", path.display());
                            }
                            entry.unpack(&filename)?;
                        } else if !quiet {
                            eprintln!("{} already exists", path.display());
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
