use std::error::Error;
use std::path::Path;
use tempfile::tempdir;
use unzipper::{FileToUnzip, FilesToUnzip};

#[test]
fn unzip() -> Result<(), Box<dyn Error>> {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    let output_dir = tempdir()?;
    let output = output_dir.path();

    let cat = FileToUnzip::new(
        tests_dir.join("archives/cat.zip"),
        output.join("cat"),
    );
    let dog = FileToUnzip::new(
        tests_dir.join("archives/dog.zip"),
        output.join("dog"),
    );
    let mice = FileToUnzip::new(tests_dir.join("archives/mice.zip"), output);

    let to_unzip = FilesToUnzip::from(vec![cat.clone(), dog.clone(), mice.clone()]);

    futures::executor::block_on(to_unzip.unzip())?;

    assert!(cat.output().exists());
    assert!(cat.output().is_dir());
    assert!(cat.output().join("cat.txt").exists());
    assert!(cat.output().join("cat.txt").is_file());
    assert!(dog.output().exists());
    assert!(dog.output().is_dir());
    assert!(dog.output().join("dog.txt").exists());
    assert!(dog.output().join("dog.txt").is_file());

    assert!(mice.output().join("mice").exists());
    assert!(mice.output().join("mice").is_dir());
    assert!(mice.output().join("mice").join("jerry.txt").exists());
    assert!(mice.output().join("mice").join("jerry.txt").is_file());
    assert!(mice.output().join("mice").join("cherie.txt").exists());
    assert!(mice.output().join("mice").join("cherie.txt").is_file());

    output_dir.close()?;
    Ok(())
}
