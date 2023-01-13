use crate::types::short_hash;

use super::*;

fn create_object_from_mermaid(
    code: &str,
    fragment_path: impl AsRef<Path>,
    chapter_number: &str,
    counter: usize,
) -> Result<PathBuf> {
    let mmdc = which::which("mmdc")?;
    let fragment_path = fragment_path.as_ref();

    // FIXME should be SVG, but the ouput produces the following issue with cmark2svg
    // Message:  called `Result::unwrap()` on an `Err` value: Svg(ParsingFailed(NoRootNode))
    // Location: projects/cmark2tex/src/lib.rs:275

    // Make it unique by content hash
    let code_hash = short_hash(code);

    const FILEFMT: &str = "pdf";
    let filename = PathBuf::from(format!(
        "fishextract_{}_{}__{}.{}",
        chapter_number.replace('.', "_"),
        counter,
        code_hash,
        FILEFMT
    ));
    let dest = fragment_path.join(&filename);

    // only use dest for actually file usage, but only ref by filename, we are in `src` when it's going to be rendered
    if dest.exists() {
        log::debug!(
            "Fishextract already present, unique by hash, skipping re-generation of {}",
            dest.display()
        );
        return Ok(filename);
    }

    log::debug!("Generating mermaid replacement file {}", dest.display());

    let mut child = std::process::Command::new(mmdc)
        .arg(format!("--outputFormat={}", FILEFMT))
        .arg(format!("--output={}", dest.display()))
        .arg("--width=700")
        .arg("--height=700")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    // FIXME make this simpler
    let code = code.to_owned();
    let mut stdin = child.stdin.take().expect("Has stdin. qed");
    let j = std::thread::spawn(move || {
        stdin.write(code.as_bytes())?;
        Ok::<_, crate::errors::Error>(())
    });
    // let mut stdout = child.stdout.expect("Has stdout. qed");
    // let mut buf = String::with_capacity(8192);
    // stdout.read_to_string(&mut buf)?;

    j.join().unwrap()?;

    let status = child.wait()?;
    if status.success() {
        Ok(filename)
    } else {
        Err(Error::MermaidSubprocess(status))
    }
}

/// Replaces the content of the cmark file where codeblocks tagged with `mermaid`
/// so for
pub fn replace_mermaid_charts(
    source: &str,
    chapter_number: &str,
    chapter_name: &str,
    chapter_path: impl AsRef<Path>,
    fragment_path: impl AsRef<Path>,
    renderer: SupportedRenderer,
    used_fragments: &mut Vec<PathBuf>,
) -> Result<String> {
    let chapter_path = chapter_path.as_ref();
    match renderer {
        // html can just fine deal with it
        SupportedRenderer::Html => return Ok(source.to_owned()),
        _ => {}
    }

    let fragment_path = fragment_path.as_ref();

    use pulldown_cmark::*;

    let mut buf = String::with_capacity(source.len());

    #[derive(Debug, Default)]
    struct State {
        is_mermaid_block: bool,
        counter: usize,
    }

    let mut events = vec![];
    let mut state = State::default();
    for (event, _offset) in Parser::new_ext(&source, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(ref kind)) => match kind {
                CodeBlockKind::Fenced(s) if s.as_ref() == "mermaid" => {
                    state.counter += 1;
                    state.is_mermaid_block = true;
                    continue;
                }
                _ => {}
            },
            Event::End(Tag::CodeBlock(ref kind)) => match kind {
                CodeBlockKind::Fenced(s) if s.as_ref() == "mermaid" => {
                    state.is_mermaid_block = false;
                    continue;
                }
                _ => {}
            },

            Event::Text(ref code) | Event::Code(ref code) => {
                if state.is_mermaid_block {
                    let image_path = create_object_from_mermaid(
                        code.as_ref(),
                        fragment_path,
                        chapter_number,
                        state.counter,
                    )?;
                    used_fragments.push(image_path.clone());

                    log::info!(
                        "🧜 Replacing mermaid graph #{} in chapter \"{} - {}\" in file {} with pdf {}",
                        state.counter,
                        chapter_number, chapter_name, chapter_path.display(),
                        image_path.display()
                    );

                    let desc = CowStr::from(format!(
                        "Chapter {}, Graphic {}",
                        chapter_number, state.counter
                    ));
                    let title = desc.clone();
                    let inject = Tag::Image(
                        LinkType::Inline,
                        image_path.display().to_string().into(),
                        title,
                    );

                    events.push(Event::Start(inject.clone()));
                    events.push(Event::Text(desc));
                    events.push(Event::End(inject));
                    events.push(Event::SoftBreak);
                    continue;
                }
            }
            _ => {}
        }
        events.push(event);
    }

    pulldown_cmark_to_cmark::cmark(events.into_iter(), &mut buf).map_err(Error::CommonMarkGlue)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
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
}
