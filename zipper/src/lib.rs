mod error;

use crate::error::Result;

use path_slash::PathExt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone)]
pub struct ToZip {
    archive: PathBuf,
    what: Vec<WhatToZip>,
}

#[derive(Debug, Clone)]
pub enum WhatToZip {
    File(PathBuf),
    Folder(PathBuf),
}

impl ToZip {
    pub fn new(archive: impl Into<PathBuf>) -> Self {
        Self {
            archive: archive.into(),
            what: vec![],
        }
    }

    pub fn file(mut self, file: impl Into<PathBuf>) -> Self {
        self.what.push(WhatToZip::File(file.into()));
        self
    }

    pub fn folder(mut self, folder: impl Into<PathBuf>) -> Self {
        self.what.push(WhatToZip::Folder(folder.into()));
        self
    }

    pub fn archive(&self) -> &Path {
        self.archive.as_path()
    }

    pub fn zip(&self) -> Result<()> {
        let archive = std::fs::File::create(self.archive()).unwrap();
        let mut zip = zip::ZipWriter::new(archive);

        let zip_options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for what in self.what.iter() {
            match what {
                WhatToZip::File(file) => {
                    zip_file(&mut zip, file, zip_options)?;
                }
                WhatToZip::Folder(folder) => {
                    zip_folder(&mut zip, folder, zip_options)?;
                }
            }
        }

        Ok(())
    }
}

fn zip_folder<F: std::io::Write + std::io::Seek>(
    zip: &mut ZipWriter<F>,
    src_dir: impl AsRef<Path>,
    zip_options: FileOptions,
) -> Result<()> {
    let src_dir = src_dir.as_ref();

    let walk_dir = WalkDir::new(src_dir);
    let it = walk_dir.into_iter();

    let mut buffer = Vec::new();
    for entry in it {
        let entry = entry?;
        let path = entry.path();

        let name = path
            .strip_prefix(src_dir.parent().expect("Could not get a parent folder"))
            .unwrap();
        let name = name
            .to_str()
            .expect("Could not convert file name to Unicode");

        // zip requires that folder separator is /, even on windows
        let name = if cfg!(windows) {
            Path::new(name).to_slash().unwrap_or(name.to_owned())
        } else {
            name.to_owned()
        };

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            let mut file_options = zip_options.clone();
            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let unix_mode: u32 = std::fs::metadata(path)?.permissions().mode();
                file_options = file_options.unix_permissions(unix_mode);
            }

            zip.start_file(name, file_options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.len() != 0 {
            zip.add_directory(name, zip_options)?;
        }
    }

    Ok(())
}

fn zip_file<F: std::io::Write + std::io::Seek>(
    zip: &mut ZipWriter<F>,
    file: impl AsRef<Path>,
    mut zip_options: FileOptions,
) -> Result<()> {
    let file = file.as_ref();
    let name = file
        .file_name()
        .expect("Could not get file name")
        .to_str()
        .expect("Could not convert file name to Unicode");

    // Get and Set permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let unix_mode: u32 = std::fs::metadata(file)?.permissions().mode();
        zip_options = zip_options.unix_permissions(unix_mode);
    }

    zip.start_file(name, zip_options)?;

    let mut f = File::open(file)?;
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer)?;
    zip.write_all(buffer.as_slice())?;
    buffer.clear();

    Ok(())
}
