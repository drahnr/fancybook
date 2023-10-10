extern crate html2md;
extern crate regex;
use html2md::parse_html;
#[macro_use]
extern crate log;
extern crate env_logger;

use fs_err as fs;
use inflector::cases::kebabcase::to_kebab_case;
use mathyank::Content;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag};
use regex::Regex;
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use resvg::{self, render};
use std::default::Default;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Range;
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
pub fn cmark_to_tex(
    cmark: impl AsRef<str>,
    dest: impl AsRef<Path>,
    asset_lookup_paths: &[PathBuf],
) -> Result<String> {
    use mathyank::*;

    let source = cmark.as_ref();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_MATH);

    // run math first, it might include any of the other characters
    let mi = dollar_split_tags_iter(source);
    let mi = iter_over_dollar_encompassed_blocks(source, mi).collect::<Vec<_>>();

    let mut equation_items = Vec::with_capacity(128);

    let source: String = mi
        .into_iter()
        .map(|tagged| {
            match tagged {
                Tagged::Replace(content) => {
                    let idx = equation_items.len();
                    equation_items.push(content);
                    // track all math equations by idx, the index is the ref into the stack
                    // we hijack the experimental math
                    let s = format!("${}$", idx);
                    s
                }
                Tagged::Keep(content) => content.s.to_owned(),
            }
        })
        .collect();

    let parser = Parser::new_ext(source.as_str(), options);
    let parser = parser.into_offset_iter();

    parser_to_tex(
        parser,
        equation_items.as_slice(),
        dest.as_ref(),
        asset_lookup_paths,
    )
}

/// Takes a pulldown_cmark::Parser or any iterator containing `pulldown_cmark::Event` and transforms it to a string
///
/// `asset_path` is the prefix path to be used for paths, could be empty, relative of absolute. Relative to the cwd.
/// Must remain a relative path!
pub fn parser_to_tex<'a, P>(
    parser: P,
    equations: &[Content<'a>],
    dest: &Path,
    asset_lookup_paths: &[PathBuf],
) -> Result<String>
where
    P: 'a + Iterator<Item = (Event<'a>, Range<usize>)>,
{
    //env_logger::init();
    let mut output = String::new();

    let header_value = "";

    let mut current: CurrentType = CurrentType {
        event_type: EventType::Text,
    };
    let mut cells = 0;

    let mut is_code = false;

    for (event, _) in parser {
        log::trace!("Event: {:?}", &event);
        match event {
            Event::Start(Tag::Heading(level, _maybe, _vec)) => {
                current.event_type = EventType::Header;
                output.push('\n');
                match level {
                    // -1 => output.push_str("part{"),
                    HeadingLevel::H1 => output.push_str(r"\chapter{"),
                    HeadingLevel::H2 => output.push_str(r"\section{"),
                    HeadingLevel::H3 => output.push_str(r"\subsection{"),
                    HeadingLevel::H4 => output.push_str(r"\subsubsection{"),
                    HeadingLevel::H5 => output.push_str(r"\paragraph{"),
                    HeadingLevel::H6 => output.push_str(r"\subparagraph{"),
                }
            }
            Event::End(Tag::Heading(_, _, _)) => {
                output.push_str("}\n");
                output.push_str(r"\label{");
                output.push_str(&header_value);
                output.push_str("}\n");

                output.push_str(r"\label{");
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
                    for entry in WalkDir::new(dest).into_iter().filter_map(|e| e.ok()) {
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
                    output.push('\n');
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
                let path = PathBuf::from(path_str);

                let mut x = Err(Error::LookupDirs(path.clone(), asset_lookup_paths.to_vec()));
                for asset_path in asset_lookup_paths.iter() {
                    let path = asset_path.join(&path);
                    if !path.is_file() {
                        log::debug!(
                            "File {} not found in asset path: {}",
                            path.display(),
                            asset_path.display()
                        );
                        continue;
                    }
                    x = Ok(path);
                    break;
                }
                let resolved_asset_path = x?;

                let resolved_asset_path = if let Some("svg") = get_extension(&resolved_asset_path) {
                    let path_png = dest.join("converted").join(&path).with_extension("png");
                    log::debug!(
                        "Replacing svg with png: {} -> {} where {}",
                        resolved_asset_path.display(),
                        path_png.display(),
                        std::env::current_dir().unwrap().to_str().unwrap()
                    );

                    // create output directories, just in case.
                    fs::create_dir_all(path_png.parent().unwrap())?;

                    let img = svg2png(&resolved_asset_path)?;
                    log::info!("Converting svg: {}", path.display());
                    log::info!("... to png: {}", path_png.display());

                    fs::write(dbg!(&path_png), img)?;
                    path_png
                } else {
                    resolved_asset_path
                };
                let resolved_asset_path = resolved_asset_path.to_str().unwrap();

                let caption = &*title;
                output.push_str(
                    format!(
                        r###"\begin{{figure}}%
\centering%
\includegraphics[width=\textwidth]{{{resolved_asset_path}}}%
\caption{{{caption}}}%
\end{{figure}}%
"###
                    )
                    .as_str(),
                );
            }

            Event::Start(Tag::Item) => output.push_str("\\item "),
            Event::End(Tag::Item) => output.push_str("\n"),

            Event::Start(Tag::CodeBlock(kind)) => {
                is_code = true;
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
                is_code = false;
            }

            Event::Math(_math_display, math) => {
                use mathyank::*;
                // don't care if ref or not, it all sits on the stack
                // lookup the item by index
                let idx = usize::from_str_radix(&math, 10)?;

                // there won't be any maths that we didnt stack
                assert!(
                    idx < equations.len(),
                    "Index is {idx} but must be less than length"
                );
                let content = &equations[idx];
                let item = Item::try_from(content)?;
                let addendum = match item {
                    Item::Block(BlockEqu {
                        title: _,
                        refer,
                        kind: _,
                        content,
                    }) => {
                        let math = content.trimmed();
                        let math = math.as_str().trim();
                        if let Some(refer) = refer {
                            format!(
                                r###"
\begin{{align}}%
\label{{{refer}}}%
{math}
\end{{align}}"###
                            )
                        } else {
                            format!(
                                r###"
\begin{{align}}%
{math}
\end{{align}}"###
                            )
                        }
                    }
                    Item::Inline(Inline::Equation(InlineEqu { content })) => {
                        format!("${}$", content.trimmed().as_str())
                    }
                    Item::Inline(Inline::Reference(Reference {
                        refere,
                        ref_kind: _,
                    })) => {
                        format!(r"\eqref{{{refere}}}")
                    }
                };
                output.push_str(&addendum);
            }

            Event::Code(t) => {
                output.push_str(r"\lstinline|");
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
                output.push_str(cmark_to_tex(parsed, dest, &asset_lookup_paths)?.as_str());
                current.event_type = EventType::Text;
            }
            Event::Text(text) => {
                if is_code {
                    // do not escape anything in code blocks
                    output.push_str(&text);
                } else {
                    // general text
                    let text = text.as_ref();
                    let text = text
                        .replace('#', r"\#")
                        .replace('%', r"\%")
                        .replace('&', r"\&")
                        .replace('^', r"\^");

                    output.push_str(&text);
                }
            }

            Event::SoftBreak => {
                output.push('\n');
            }

            Event::HardBreak => {
                output.push('\n');
                output.push_str(r"\\");
            }

            _ => (),
        }
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
pub fn get_extension(filename: &std::path::Path) -> Option<&str> {
    filename.extension().and_then(OsStr::to_str)
}

#[cfg(test)]
mod tests;
