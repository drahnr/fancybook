extern crate html2md;
extern crate regex;
use html2md::parse_html;
#[macro_use]
extern crate log;
extern crate env_logger;

use fs_err as fs;
use inflector::cases::kebabcase::to_kebab_case;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, MathDisplay, Options, Parser, Tag};
use regex::Regex;
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::{self, render};
use std::default::Default;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;
use walkdir::WalkDir;

pub mod error;
pub use self::error::*;

/// Used to keep track of current pulldown_cmark "event".
/// TODO: Is there a native pulldown_cmark method to do this?
#[derive(Debug)]
enum EventType {
    Code,
    Emphasis,
    Header,
    Html,
    Strong,
    Table,
    TableHead,
    Text,
}

pub struct CurrentType {
    event_type: EventType,
}

/// Converts markdown string to tex string.
pub fn cmark_to_tex(cmark: impl AsRef<str>, asset_path: impl AsRef<Path>) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_MATH);

    let parser = Parser::new_ext(cmark.as_ref(), options);

    parser_to_tex(parser, asset_path.as_ref())
}

/// Takes a pulldown_cmark::Parser or any iterator containing `pulldown_cmark::Event` and transforms it to a string
///
/// `asset_path` is the prefix path to be used for paths, could be empty, relative of absolute. Relative to the cwd.
/// Must remain a relative path!
pub fn parser_to_tex<'a, P>(parser: P, asset_path: &Path) -> Result<String>
where
    P: 'a + Iterator<Item = Event<'a>>,
{
    //env_logger::init();
    let mut output = String::new();

    let header_value = "";

    let mut current: CurrentType = CurrentType {
        event_type: EventType::Text,
    };
    let mut cells = 0;

    let mut buffer = String::new();

    for event in parser {
        log::trace!("Event: {:?}", event);
        match event {
            Event::Start(Tag::Heading(level, _maybe, _vec)) => {
                current.event_type = EventType::Header;
                output.push_str("\n");
                output.push_str("\\");
                match level {
                    // -1 => output.push_str("part{"),
                    HeadingLevel::H1 => output.push_str("chapter{"),
                    HeadingLevel::H2 => output.push_str("section{"),
                    HeadingLevel::H3 => output.push_str("subsection{"),
                    HeadingLevel::H4 => output.push_str("subsubsection{"),
                    HeadingLevel::H5 => output.push_str("paragraph{"),
                    HeadingLevel::H6 => output.push_str("subparagraph{"),
                }
            }
            Event::End(Tag::Heading(_, _, _)) => {
                output.push_str("}\n");
                output.push_str("\\");
                output.push_str("label{");
                output.push_str(&header_value);
                output.push_str("}\n");

                output.push_str("\\");
                output.push_str("label{");
                output.push_str(&to_kebab_case(&header_value));
                output.push_str("}\n");
            }
            Event::Start(Tag::Emphasis) => {
                current.event_type = EventType::Emphasis;
                output.push_str("\\emph{");
            }
            Event::End(Tag::Emphasis) => output.push_str("}"),

            Event::Start(Tag::Strong) => {
                current.event_type = EventType::Strong;
                output.push_str("\\textbf{");
            }
            Event::End(Tag::Strong) => output.push_str("}"),

            Event::Start(Tag::List(None)) => output.push_str("\\begin{itemize}\n"),
            Event::End(Tag::List(None)) => output.push_str("\\end{itemize}\n"),

            Event::Start(Tag::List(Some(_))) => output.push_str("\\begin{enumerate}\n"),
            Event::End(Tag::List(Some(_))) => output.push_str("\\end{enumerate}\n"),

            Event::Start(Tag::Paragraph) => {
                output.push_str("\n");
            }

            Event::End(Tag::Paragraph) => {
                // ~ adds a space to prevent
                // "There's no line here to end" error on empty lines.
                output.push_str(r"~\\");
                output.push_str("\n");
            }

            Event::Start(Tag::Link(_, url, _)) => {
                // URL link (e.g. "https://nasa.gov/my/cool/figure.png")
                if url.starts_with("http") {
                    output.push_str("\\href{");
                    output.push_str(&*url);
                    output.push_str("}{");
                // local link (e.g. "my/cool/figure.png")
                } else {
                    output.push_str("\\hyperref[");
                    let mut found = false;

                    // iterate through `src` directory to find the resource.
                    for entry in WalkDir::new(asset_path).into_iter().filter_map(|e| e.ok()) {
                        let _path = entry.path().to_str().unwrap();
                        let _url = &url.clone().into_string().replace("../", "");
                        if _path.ends_with(_url) {
                            debug!("{}", entry.path().display());
                            debug!("URL: {}", url);

                            let file = match fs::File::open(_path) {
                                Ok(file) => file,
                                Err(_) => panic!("Unable to read title from {}", _path),
                            };
                            let buffer = BufReader::new(file);

                            let title = title_string(buffer);
                            output.push_str(&title);

                            debug!("The`` title is '{}'", title);

                            found = true;
                            break;
                        }
                    }

                    if !found {
                        output.push_str(&*url.replace("#", ""));
                    }

                    output.push_str("]{");
                }
            }

            Event::End(Tag::Link(_, _, _)) => {
                output.push_str("}");
            }

            Event::Start(Tag::Table(_)) => {
                current.event_type = EventType::Table;
                let table_start = vec![
                    "\n",
                    r"\begingroup",
                    r"\setlength{\LTleft}{-20cm plus -1fill}",
                    r"\setlength{\LTright}{\LTleft}",
                    r"\begin{longtable}{!!!}",
                    r"\hline",
                    r"\hline",
                    "\n",
                ];
                for element in table_start {
                    output.push_str(element);
                    output.push_str("\n");
                }
            }

            Event::Start(Tag::TableHead) => {
                current.event_type = EventType::TableHead;
            }

            Event::End(Tag::TableHead) => {
                output.truncate(output.len() - 2);
                output.push_str(r"\\");
                output.push_str("\n");

                output.push_str(r"\hline");
                output.push_str("\n");

                // we presume that a table follows every table head.
                current.event_type = EventType::Table;
            }

            Event::End(Tag::Table(_)) => {
                let table_end = vec![
                    r"\arrayrulecolor{black}\hline",
                    r"\end{longtable}",
                    r"\endgroup",
                    "\n",
                ];

                for element in table_end {
                    output.push_str(element);
                    output.push_str("\n");
                }

                let mut cols = String::new();
                for _i in 0..cells {
                    cols.push_str(&format!(
                        r"C{{{width}\textwidth}} ",
                        width = 1. / cells as f64
                    ));
                }
                output = output.replace("!!!", &cols);
                cells = 0;
                current.event_type = EventType::Text;
            }

            Event::Start(Tag::TableCell) => match current.event_type {
                EventType::TableHead => {
                    output.push_str(r"\bfseries{");
                }
                _ => (),
            },

            Event::End(Tag::TableCell) => {
                match current.event_type {
                    EventType::TableHead => {
                        output.push_str(r"}");
                        cells += 1;
                    }
                    _ => (),
                }

                output.push_str(" & ");
            }

            Event::Start(Tag::TableRow) => {
                current.event_type = EventType::Table;
            }

            Event::End(Tag::TableRow) => {
                output.truncate(output.len() - 2);
                output.push_str(r"\\");
                output.push_str(r"\arrayrulecolor{lightgray}\hline");
                output.push_str("\n");
            }

            Event::Start(Tag::Image(_, path, title)) => {
                let mut path_str = path.clone().into_string();

                // if image path ends with ".svg", run it through
                // svg2png to convert to png file.
                if let Some("svg") = get_extension(&path) {
                    let path = PathBuf::from(path.as_ref());
                    let path_png = asset_path.join(&path).with_extension("png");
                    log::debug!(
                        "Replacing svg with png: {} -> {} where {}",
                        path.display(),
                        path_png.display(),
                        std::env::current_dir().unwrap().to_str().unwrap()
                    );

                    // create output directories, just in case.
                    fs::create_dir_all(path_png.parent().unwrap())?;

                    let img = svg2png(&path)?;

                    fs::write(&path_png, img)?;
                    path_str = path_png
                        .to_str()
                        .expect("Works, we just created it from a valid str. qed")
                        .to_owned();
                }

                let caption = &*title;
                let path = path_str;
                output.push_str(
                    format!(
                        r###"\begin{{figure}}
\centering
\includegraphics[width=\textwidth]{{{path}}}
\caption{{{caption}}}
\end{{figure}}
"###
                    )
                    .as_str(),
                );
            }

            Event::Start(Tag::Item) => output.push_str("\\item "),
            Event::End(Tag::Item) => output.push_str("\n"),

            Event::Start(Tag::CodeBlock(kind)) => {
                let re = Regex::new(r",.*").unwrap();
                current.event_type = EventType::Code;
                if let CodeBlockKind::Fenced(lang) = kind {
                    output.push_str("\\begin{lstlisting}[language=");
                    output.push_str(&re.replace(&lang, ""));
                    output.push_str("]\n");
                } else {
                    output.push_str("\\begin{lstlisting}\n");
                }
            }

            Event::End(Tag::CodeBlock(_)) => {
                output.push_str("\n\\end{lstlisting}\n");
                current.event_type = EventType::Text;
            }

            Event::Math(math_display, math) => {
                output.push_str(
                    match math_display {
                        MathDisplay::Block => {
                            // FIXME extract equation references
                            format!(
                                r###"
\begin{{align}}
%\label{{}}
{math}
\end{{align}}
"###
                            )
                        }
                        MathDisplay::Inline => {
                            // FIXME TODO handle all the referencing and shiat
                            format!(r###"${math}$"###)
                        }
                    }
                    .as_str(),
                );
            }

            Event::Code(t) => {
                output.push_str("\\lstinline|");
                match current.event_type {
                    EventType::Header => output
                        .push_str(&*t.replace("#", r"\#").replace("…", "...").replace("З", "3")),
                    _ => output
                        .push_str(&*t.replace("…", "...").replace("З", "3").replace("�", r"\�")),
                }
                output.push_str("|");
            }

            Event::Html(t) => {
                current.event_type = EventType::Html;
                // convert common html patterns to tex
                let parsed = parse_html(&t.into_string());
                output.push_str(cmark_to_tex(parsed, asset_path)?.as_str());
                current.event_type = EventType::Text;
            }

            Event::Text(t) => {
                buffer.push_str(&t);
            }

            Event::SoftBreak => {
                output.push('\n');
            }

            Event::HardBreak => {
                output.push_str(r"\\");
                output.push('\n');
            }

            _ => (),
        }
    }

    Ok(output)
}

/// Simple HTML parser.
///
/// Eventually I hope to use a mature HTML to tex parser.
/// Something along the lines of https://github.com/Adonai/html2md/
pub fn html2tex(html: String, current: &CurrentType) -> Result<String> {
    let mut tex = html;
    let mut output = String::new();

    // remove all "class=foo" and "id=bar".
    let re = Regex::new(r#"\s(class|id)="[a-zA-Z0-9-_]*">"#).unwrap();
    tex = re.replace(&tex, "").to_string();

    // image html tags
    if tex.contains("<img") {
        // Regex doesn't yet support look aheads (.*?), so we'll use simple pattern matching.
        // let src = Regex::new(r#"src="(.*?)"#).unwrap();
        let src = Regex::new(r#"src="([a-zA-Z0-9-/_.]*)"#).unwrap();
        let caps = src.captures(&tex).unwrap();
        let path_raw = caps.get(1).unwrap().as_str();
        let path = format!("../../{path}", path = path_raw);

        // if path ends with ".svg", run it through
        // svg2png to convert to png file.
        if let Some("svg") = get_extension(&path) {
            let orig = &path;
            let path = PathBuf::from(orig.as_str());
            let img = svg2png(&path)?;

            let path = PathBuf::from(orig.replace("../../", "")).with_extension("png");
            log::debug!("path!: {}", path.display());

            // create output directories.
            let _ = fs::create_dir_all(path.parent().unwrap());

            fs::write(&path, img)?;
        }

        match current.event_type {
            EventType::Table => {
                output.push_str(r"\begin{center}\includegraphics[width=0.2\textwidth]{")
            }
            _ => {
                output.push_str(r"\begin{center}\includegraphics[width=0.8\textwidth]{");
            }
        }

        output.push_str(path.as_str());
        output.push_str(r"}\end{center}");
        output.push_str("\n");

    // all other tags
    } else {
        match current.event_type {
            // block code
            EventType::Html => {
                tex = parse_html_description(tex);

                tex = tex
                    .replace("/>", "")
                    .replace("<code class=\"language-", "\\begin{lstlisting}")
                    .replace("</code>", r"\\end{lstlisting}")
                    .replace("<span", "")
                    .replace(r"</span>", "")
            }
            // inline code
            _ => {
                tex = tex
                    .replace("/>", "")
                    .replace("<code\n", "<code")
                    .replace("<code", r"\lstinline|")
                    .replace("</code>", r"|")
                    .replace("<span", "")
                    .replace(r"</span>", "");
            }
        }
        // remove all HTML comments.
        let re = Regex::new(r"<!--.*-->").unwrap();
        output.push_str(&re.replace(&tex, ""));
    }

    Ok(output)
}

/// Convert HTML description elements into LaTeX equivalents.
pub fn parse_html_description(tex: String) -> String {
    let descriptionized = tex;
    descriptionized
}

/// Get the title of a Markdown file.
///
/// Reads the first line of a Markdown file, strips any hashes and
/// leading/trailing whitespace, and returns the title.
/// Source: https://codereview.stackexchange.com/questions/135013/rust-function-to-read-the-first-line-of-a-file-strip-leading-hashes-and-whitesp
pub fn title_string<R>(mut rdr: R) -> String
where
    R: BufRead,
{
    let mut first_line = String::new();

    rdr.read_line(&mut first_line).expect("Unable to read line");

    // Where do the leading hashes stop?
    let last_hash = first_line
        .char_indices()
        .skip_while(|&(_, c)| c == '#')
        .next()
        .map_or(0, |(idx, _)| idx);

    // Trim the leading hashes and any whitespace
    first_line[last_hash..].trim().into()
}

/// Converts an SVG file to a PNG file.
///
/// Example: foo.svg becomes foo.svg.png
pub fn svg2png(svg_path: &Path) -> Result<Vec<u8>> {
    debug!("svg2png operating on {}", svg_path.display());
    let data = fs::read_to_string(svg_path)?;

    if data.is_empty() {
        Err(Error::Svg(
            None,
            format!("Given SVG file is empty {}", svg_path.display()),
        ))?
    }
    const MIN_BYTE_LENGTH: usize = 30;
    if data.len() < MIN_BYTE_LENGTH {
        Err(Error::Svg(
            None,
            format!(
                "Given SVG file is less than {} bytes: {}",
                MIN_BYTE_LENGTH,
                svg_path.display()
            ),
        ))?
    }

    let rtree = usvg::Tree::from_data(data.as_bytes(), &usvg::Options::default()).map_err(|e| {
        Error::Svg(
            e.into(),
            format!("Loading failure for svg {}", svg_path.display()),
        )
    })?;
    let mut pixi = Pixmap::new(rtree.size.width() as u32, rtree.size.height() as u32).unwrap();
    render(
        &rtree,
        usvg::FitTo::Original,
        resvg::tiny_skia::Transform::default(),
        pixi.as_mut(),
    )
    .unwrap();
    let img = pixi.encode_png().unwrap();
    Ok(img)
}

/// Extract extension from filename
///
/// Source:  https://stackoverflow.com/questions/45291832/extracting-a-file-extension-from-a-given-path-in-rust-idiomatically
pub fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

#[cfg(test)]
mod tests;
