mod error;

use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub use error::{Result, UnzipperError};

#[derive(Debug, Clone)]
pub struct FileToUnzip {
    archive: PathBuf,
    output: PathBuf,
}

impl FileToUnzip {
    pub fn new(archive: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self {
            archive: archive.into(),
            output: output.into(),
        }
    }

    pub fn output(&self) -> &Path {
        self.output.as_path()
    }
}

#[derive(Debug, Clone)]
pub struct FilesToUnzip {
    files: Vec<FileToUnzip>,
}

impl FilesToUnzip {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn from(files: impl IntoIterator<Item = FileToUnzip>) -> Self {
        Self {
            files: files.into_iter().collect::<Vec<FileToUnzip>>(),
        }
    }

    pub fn add(self, file_to_unzip: FileToUnzip) -> Self {
        let mut files = self.files.clone();
        files.push(file_to_unzip);
        Self { files }
    }

    pub fn maybe_add(self, file_to_unzip: Option<FileToUnzip>) -> Self {
        if let Some(file_to_unzip) = file_to_unzip {
            self.add(file_to_unzip)
        } else {
            self
        }
    }

    pub fn add_file(self, archive: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        self.add(FileToUnzip::new(archive, output))
    }

    pub fn extend(self, files_to_unzip: Self) -> Self {
        let mut files = self.files.clone();
        files.extend(files_to_unzip.files);
        Self { files }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub async fn unzip(self) -> Result<()> {
        let multibar = MultiProgress::new();
        let all_zips_pb = multibar.add(ProgressBar::new(self.files.len() as u64));

        all_zips_pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {bar:10} {pos}/{len}")
                .unwrap(),
        );
        all_zips_pb.set_message("total  ");
        all_zips_pb.tick();

        // Set up a future to iterate over tasks and run up to 2 at a time.
        let tasks = stream::iter(&self.files).enumerate().for_each_concurrent(
            Some(2),
            |(_i, file_to_unzip)| async {
                // Clone multibar and main_pb.  We will move the clones into each task.
                let multibar = multibar.clone();
                let main_pb = all_zips_pb.clone();
                let file_to_unzip = file_to_unzip.clone();

                futures::future::lazy(|_| unzip_task(file_to_unzip.clone(), multibar))
                    .await
                    .expect(format!("Failed to unzip {:?}", &file_to_unzip).as_str());

                main_pb.inc(1);
            },
        );

        // Wait for the tasks to finish.
        tasks.await;

        // Change the message on the overall progress indicator.
        all_zips_pb.finish_with_message("done");
        Ok(())
    }
}

pub fn unzip_task(file_to_unzip: FileToUnzip, multibar: MultiProgress) -> Result<()> {
    let file = std::fs::File::open(&file_to_unzip.archive)?;
    let mut archive = ZipArchive::new(file)?;

    // Create the ProgressBar with the acquired size from before
    // and add it to the multi-bar
    let progress_bar = multibar.add(ProgressBar::new(archive.len() as u64));

    // Set Style to the ProgressBar
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {percent}% - {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Set the filename as message part of the progress bar
    progress_bar.set_message(
        file_to_unzip
            .archive
            .clone()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();

        let output_path = match file.enclosed_name() {
            Some(path) => file_to_unzip.output.join(path),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            std::fs::create_dir_all(&output_path).unwrap();
        } else {
            if let Some(p) = output_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&output_path).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&output_path, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
        progress_bar.inc(1)
    }

    // Finish the progress bar to prevent glitches
    progress_bar.finish();

    Ok(())
}
