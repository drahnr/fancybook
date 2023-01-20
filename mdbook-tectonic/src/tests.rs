use super::*;
use std::path::PathBuf;

#[test]
fn test_traverse_markdown() -> Result<(), Box<dyn std::error::Error>> {
    let imagepath = Path::new("/tmp/test/images/chap/foo.png");
    // create a temporary directory in /tmp/
    fs::create_dir_all(imagepath.parent().unwrap()).expect("failure while creating testdirs");
    // touch the mock png file;
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("mdbook-latex.png"),
        imagepath,
    )?;

    let path = PathBuf::from(r"chap");
    let context = RenderContext::new(
        Path::new("/tmp/test/"),
        mdbook::book::Book::new(),
        mdbook::Config::default(),
        Path::new("/tmp/target_asset_dir/"),
    );
    let new_content = traverse_markdown(
        "![123](images/chap/foo.png)",
        &path,
        &["somewhere".into()],
        &context,
    )
    .unwrap();
    assert_eq!("![123](images/chap/foo.png)", new_content);
    let respath = Path::new("/tmp/target_asset_dir/somewhere/images/chap/foo.png");
    assert!(respath.exists());

    fs::remove_dir_all("/tmp/test")?;
    fs::remove_dir_all("/tmp/dest")?;
    Ok(())
}
