mod error;

pub use crate::error::{Result, ZipperError};

use path_slash::PathExt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::{FileOptionExtension, FileOptions, SimpleFileOptions};
use zip::ZipWriter;

#[cfg(feature = "file-matcher")]
use file_matcher::OneEntry;

#[derive(Debug, Clone)]
pub struct ToZip {
    archive: PathBuf,
    what: Vec<WhatToZip>,
}

#[derive(Debug, Clone)]
pub enum WhatToZip {
    File(PathBuf),
    Folder(PathBuf),
    #[cfg(feature = "file-matcher")]
    OneEntry(OneEntry),
}

impl ToZip {
    pub fn new(archive: impl Into<PathBuf>) -> Self {
        Self {
            archive: archive.into(),
            what: vec![],
        }
    }

    pub fn file(mut self, file: impl Into<PathBuf>) -> Self {
        self.add_file(file);
        self
    }

    pub fn folder(mut self, folder: impl Into<PathBuf>) -> Self {
        self.add_folder(folder);
        self
    }

    #[cfg(feature = "file-matcher")]
    pub fn one_entry(mut self, one_entry: OneEntry) -> Self {
        self.add_one_entry(one_entry);
        self
    }

    #[cfg(feature = "file-matcher")]
    pub fn one_entries(mut self, entries: impl IntoIterator<Item = OneEntry>) -> Self {
        for entry in entries.into_iter() {
            self.add_one_entry(entry);
        }
        self
    }

    pub fn add_file(&mut self, file: impl Into<PathBuf>) {
        self.what.push(WhatToZip::File(file.into()));
    }

    pub fn add_folder(&mut self, folder: impl Into<PathBuf>) {
        self.what.push(WhatToZip::Folder(folder.into()));
    }

    #[cfg(feature = "file-matcher")]
    pub fn add_one_entry(&mut self, one_entry: OneEntry) {
        self.what.push(WhatToZip::OneEntry(one_entry));
    }

    pub fn archive(&self) -> &Path {
        self.archive.as_path()
    }

    pub fn zip(&self) -> Result<PathBuf> {
        let archive = File::create(self.archive()).unwrap();
        let mut zip = ZipWriter::new(archive);

        let zip_options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for what in self.what.iter() {
            match what {
                WhatToZip::File(file) => {
                    zip_file(&mut zip, file, zip_options)?;
                }
                WhatToZip::Folder(folder) => {
                    zip_folder(&mut zip, folder, zip_options)?;
                }
                #[cfg(feature = "file-matcher")]
                WhatToZip::OneEntry(one_entry) => {
                    let path = one_entry.as_path_buf()?;
                    if path.is_file() {
                        zip_file(&mut zip, path, zip_options)?;
                    } else if path.is_dir() {
                        zip_folder(&mut zip, path, zip_options)?;
                    } else {
                        Err(ZipperError::UnknownEntryType(path))?
                    }
                }
            }
        }

        zip.finish()?;

        Ok(self.archive.clone())
    }
}

fn zip_folder<F: Write + std::io::Seek, T: FileOptionExtension + Clone>(
    zip: &mut ZipWriter<F>,
    src_dir: impl AsRef<Path>,
    zip_options: FileOptions<T>,
) -> Result<()> {
    let src_dir = src_dir.as_ref();
    if !src_dir.exists() {
        return Err(ZipperError::FolderDoesNotExist(src_dir.to_owned()));
    }

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
            Path::new(name).to_slash().unwrap_or(name.into())
        } else {
            name.into()
        };

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            #[allow(unused_mut)]
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
            zip.add_directory(name, zip_options.clone())?;
        }
    }

    Ok(())
}

#[allow(unused_mut)]
fn zip_file<F: Write + std::io::Seek, T: FileOptionExtension>(
    zip: &mut ZipWriter<F>,
    file: impl AsRef<Path>,
    mut zip_options: FileOptions<T>,
) -> Result<()> {
    let file = file.as_ref();

    if !file.exists() {
        return Err(ZipperError::FileDoesNotExist(file.to_owned()));
    }

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
