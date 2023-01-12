use super::*;
use std::fs::OpenOptions;
use std::path::PathBuf;

#[test]
fn test_traverse_markdown() {
    let imgpath = Path::new("/tmp/test/src/chap/xyz.png");
    // create a temporary directory in /tmp/
    fs::create_dir_all(imgpath.parent().unwrap()).expect("failure while creating testdirs");
    // touch the mock png file
    let _ = match OpenOptions::new().create(true).write(true).open(imgpath) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    };
    let content = "![123](./xyz.png)";
    let path = PathBuf::from(r"chap/");
    let context = RenderContext::new(
        Path::new("/tmp/test/"),
        mdbook::book::Book::new(),
        mdbook::Config::default(),
        Path::new("/tmp/dest/"),
    );
    let new_content = traverse_markdown(content, &path, &context).unwrap();
    assert_eq!("![123](images/chap/xyz.png)", new_content);
    let respath = Path::new("/tmp/dest/images/chap/xyz.png");
    assert!(respath.exists());

    fs::remove_dir_all("/tmp/test").unwrap();
    fs::remove_dir_all("/tmp/dest").unwrap();
}
