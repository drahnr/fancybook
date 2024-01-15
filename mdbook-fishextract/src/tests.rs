use assert_matches::assert_matches;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::env::temp_dir;

use super::*;

#[test]
fn gen_mermaid_svg_and_replace() {
    let dest = temp_dir().join(format!("mdboff"));
    fs::create_dir_all(&dest).unwrap();
    let adjusted = replace_mermaid_charts(
        r#"
```mermaid
graph
A-->B
```
"#,
        "1.2.3".into(),
        "testchap".into(),
        PathBuf::new(),
        dest,
        SupportedRenderer::Markdown,
        &mut Vec::new(),
        &[],
    )
    .unwrap();

    let mut iter = Parser::new_ext(&adjusted, Options::all()).into_iter();

    let _ = iter.next();
    assert_matches!(dbg!(iter.next()), Some(Event::Start(Tag::Image(_, _, _))));
    assert_matches!(iter.next(), Some(Event::Text(s)) => {
        assert!(s.contains("1.2.3"));
    });
    assert_matches!(iter.next(), Some(Event::End(Tag::Image(_, _, _))));
}
