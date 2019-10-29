//! Mocks for testing

pub mod clitools;
pub mod dist;
pub mod topical_doc_data;

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

// Mock of the on-disk structure of rust-installer installers
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MockInstallerBuilder {
    pub components: Vec<MockComponentBuilder>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MockComponentBuilder {
    pub name: String,
    pub files: Vec<MockFile>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MockFile {
    path: String,
    contents: Contents,
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum Contents {
    File(MockContents),
    Dir(Vec<(&'static str, MockContents)>),
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct MockContents {
    contents: Arc<Vec<u8>>,
    executable: bool,
}

impl MockInstallerBuilder {
    pub fn build(&self, path: &Path) {
        for component in &self.components {
            // Update the components file
            let comp_file = path.join("components");
            let mut comp_file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(comp_file.clone())
                .unwrap();
            writeln!(comp_file, "{}", component.name).unwrap();

            // Create the component directory
            let component_dir = path.join(&component.name);
            if !component_dir.exists() {
                fs::create_dir(&component_dir).unwrap();
            }

            // Create the component files and manifest
            let mut manifest = File::create(component_dir.join("manifest.in")).unwrap();
            for file in component.files.iter() {
                match file.contents {
                    Contents::Dir(_) => {
                        writeln!(manifest, "dir:{}", file.path).unwrap();
                    }
                    Contents::File(_) => {
                        writeln!(manifest, "file:{}", file.path).unwrap();
                    }
                }
                file.build(&component_dir);
            }
        }

        let mut ver = File::create(path.join("rust-installer-version")).unwrap();
        writeln!(ver, "3").unwrap();
    }
}

impl MockFile {
    pub fn new<S: Into<String>>(path: S, contents: &[u8]) -> MockFile {
        MockFile::_new(path.into(), Arc::new(contents.to_vec()))
    }

    pub fn new_arc<S: Into<String>>(path: S, contents: Arc<Vec<u8>>) -> MockFile {
        MockFile::_new(path.into(), contents)
    }

    fn _new(path: String, contents: Arc<Vec<u8>>) -> MockFile {
        MockFile {
            path,
            contents: Contents::File(MockContents {
                contents,
                executable: false,
            }),
        }
    }

    pub fn new_dir(path: &str, files: &[(&'static str, &'static [u8], bool)]) -> MockFile {
        MockFile {
            path: path.to_string(),
            contents: Contents::Dir(
                files
                    .iter()
                    .map(|&(name, data, exe)| {
                        (
                            name,
                            MockContents {
                                contents: Arc::new(data.to_vec()),
                                executable: exe,
                            },
                        )
                    })
                    .collect(),
            ),
        }
    }

    pub fn executable(mut self, exe: bool) -> Self {
        if let Contents::File(c) = &mut self.contents {
            c.executable = exe;
        }
        self
    }

    pub fn build(&self, path: &Path) {
        let path = path.join(&self.path);
        match self.contents {
            Contents::Dir(ref files) => {
                for &(ref name, ref contents) in files {
                    let fname = path.join(name);
                    contents.build(&fname);
                }
            }
            Contents::File(ref contents) => contents.build(&path),
        }
    }
}

impl MockContents {
    fn build(&self, path: &Path) {
        let dir_path = path.parent().unwrap().to_owned();
        fs::create_dir_all(dir_path).unwrap();
        File::create(&path)
            .unwrap()
            .write_all(&self.contents)
            .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if self.executable {
                let mut perm = fs::metadata(path).unwrap().permissions();
                perm.set_mode(0o755);
                fs::set_permissions(path, perm).unwrap();
            }
        }
    }
}

#[cfg(windows)]
pub fn get_path() -> Option<String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .unwrap();

    environment.get_value("PATH").ok()
}

#[cfg(windows)]
pub fn restore_path(p: Option<String>) {
    use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .unwrap();

    if let Some(p) = p.as_ref() {
        let reg_value = RegValue {
            bytes: string_to_winreg_bytes(&p),
            vtype: RegType::REG_EXPAND_SZ,
        };
        environment.set_raw_value("PATH", &reg_value).unwrap();
    } else {
        let _ = environment.delete_value("PATH");
    }

    fn string_to_winreg_bytes(s: &str) -> Vec<u8> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let v: Vec<u16> = OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let ptr = v.as_ptr().cast::<u8>();
        let len = v.len() * 2;
        unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec()
    }
}

#[cfg(unix)]
pub fn get_path() -> Option<String> {
    None
}

#[cfg(unix)]
pub fn restore_path(_: Option<String>) {}
