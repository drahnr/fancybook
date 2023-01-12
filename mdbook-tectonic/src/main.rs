use cmark2tex::cmark_to_tex;
use color_eyre::eyre::bail;
use fs::OpenOptions;
use fs_err as fs;
use mdbook::book::BookItem;
use mdbook::renderer::RenderContext;
use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag};
use std::io::{self, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
mod tests;

// config definition.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LatexConfig {
    // Chapters that will not be exported.
    pub ignores: Vec<String>,

    // Output latex file.
    pub latex: bool,

    // Output PDF.
    pub pdf: bool,

    // Output markdown file.
    pub markdown: bool,

    // Use user's LaTeX template file instead of default (template.tex).
    pub custom_template: Option<String>,

    // Date to be used in the LaTeX \date{} macro
    #[serde(default = "today")]
    pub date: String,
}

fn today() -> String {
    r#"\today"#.to_owned()
}

impl Default for LatexConfig {
    fn default() -> Self {
        Self {
            ignores: Default::default(),
            latex: true,
            pdf: true,
            markdown: true,
            custom_template: None,
            date: today(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Failed to parse STDIN as `RenderContext` JSON: {0:?}")]
    MdBook(mdbook::errors::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    use env_logger::Builder;
    use log::LevelFilter;
    let mut builder = Builder::from_default_env();
    builder
        .filter(None, LevelFilter::Debug)
        .filter(Some("cmark2tex"), LevelFilter::Warn)
        .init();

    let stdin = BufReader::new(io::stdin());

    // Get markdown source from the mdbook command via stdin
    let ctx = RenderContext::from_json(stdin).map_err(Error::MdBook)?;

    let compiled_against = semver::VersionReq::parse(mdbook::MDBOOK_VERSION)?;
    let running_against = semver::Version::parse(ctx.version.as_str())?;
    if !compiled_against.matches(&running_against) {
        // We should probably use the `semver` crate to check compatibility
        // here...
        log::warn!(
            "Warning: The {} output was built against version {} of mdbook, \
             but we're being called from version {}",
            "tectonic",
            mdbook::MDBOOK_VERSION,
            ctx.version
        );
    }

    log::debug!(
        "mdbook-tectonic called from {}!",
        std::env::current_dir().unwrap().display()
    );

    // Get configuration options from book.toml.
    let cfg: LatexConfig = ctx
        .config
        .get_deserialized_opt("output.latex")
        .expect("Error reading \"output.latex\" configuration")
        .unwrap_or_default();

    // Read book's config values (title, authors).
    let title = ctx
        .config
        .book
        .title
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("<Unknown Title>");
    let authors = ctx.config.book.authors.join(" \\and ");
    let date = cfg.date.clone();

    // Copy template data into memory.
    let mut template = if let Some(custom_template) = cfg.custom_template {
        let mut custom_template_path = ctx.root.clone();
        custom_template_path.push(custom_template);
        fs::read_to_string(custom_template_path)?
    } else {
        include_str!("template.tex").to_string()
    };

    // Add title and author information.
    template = template.replace(r"\title{}", &format!("\\title{{{}}}", title));
    template = template.replace(r"\author{}", &format!("\\author{{{}}}", authors));
    template = template.replace(r"\date{}", &format!("\\date{{{}}}", date));

    let mut latex = String::new();

    // Iterate through markdown source and push the chapters onto one single string.
    let mut content = String::new();
    for item in ctx.book.iter() {
        // Iterate through each chapter.
        if let BookItem::Chapter(ref ch) = *item {
            if cfg.ignores.contains(&ch.name) {
                continue;
            }

            // Add chapter path to relative links.
            content.push_str(&traverse_markdown(
                &ch.content,
                ch.path.as_ref().unwrap().parent().unwrap(),
                &ctx,
            )?);
        }
    }

    // println!("{}", content);
    if cfg.markdown {
        // Output markdown file.
        output_markdown(".md", title, &content, &ctx.destination)?;
    }

    if cfg.latex || cfg.pdf {
        // convert markdown data to LaTeX
        latex.push_str(&cmark_to_tex(content)?);

        // Insert new LaTeX data into template after "%% mdbook-tectonic begin".
        const BEGIN: &str = "mdbook-tectonic begin";
        let pos = if let Some(pos) = template.find(&BEGIN) {
            pos
        } else {
            bail!("Missing injection point `%% {}` in tex template", BEGIN);
        } + BEGIN.len();

        template.insert_str(pos, &latex);

        if cfg.latex {
            // Output latex file.
            output_markdown(".tex", title, &template, &ctx.destination)?;
        }

        // Output PDF file.
        if cfg.pdf {
            // let mut input = tempfile::NamedTempFile::new()?;
            // input.write(template.as_bytes())?;

            // Write PDF with tectonic.
            let cwd = std::env::current_dir()?;
            println!("Writing PDF to {} with Tectonic...", cwd.display());
            // FIXME launch tectonic process
            let tectonic = which::which("tectonic")?;
            let mut child = std::process::Command::new(tectonic)
                .arg("--outfmt=pdf")
                .arg(format!("-o={}", cwd.display()))
                .arg("-")
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            {
                let mut tectonic_stdin = child.stdin.as_mut().unwrap();
                let mut tectonic_stdin = std::io::BufWriter::new(&mut tectonic_stdin);
                tectonic_stdin.write(template.as_bytes())?;
            }
            if let Some(retval) = child.wait()?.code() {
                if retval != 0 {
                    bail!("Subprocess `tectonic` terminated with exit code {}", retval)
                }
            } else {
                bail!("Failed to launch subprocess `tectonic`")
            }
        }
    }

    Ok(())
}

/// Output plain text file.
///
/// Used for writing markdown and latex data to files.
fn output_markdown<P: AsRef<Path>>(
    extension: &str,
    filename: &str,
    data: &str,
    destination: P,
) -> Result<(), io::Error> {
    // the title might contain a lot of stuff, so limit it to sane chars
    let re = regex::Regex::new("[^A-Za-z0-9_-]").expect("Parses just fine. qed");
    let filename = str::replace(filename, move |c: char| re.is_match(&c.to_string()), "");

    let mut path = PathBuf::from(filename);
    path.set_extension(extension);

    // Create output directory/file.
    fs::create_dir_all(destination)?;

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

/// This Function parses the markdown file, alters some elements and writes it back to markdown.
///
/// Changes done:
///   * change image paths to be relative to images
///   * copy the image files into the images directory in the target directory
fn traverse_markdown(
    content: &str,
    chapter_path: &Path,
    context: &RenderContext,
) -> std::io::Result<String> {
    let parser = Parser::new_ext(content, Options::all());
    let parser = parser
        .map(|event| {
            Ok(match event {
                Event::Start(Tag::Image(link_type, path, title)) => {
                    //Event::Start(Tag::Image(link_type, imagepathcowstr, title))
                    Event::Start(parse_image_tag(
                        link_type,
                        path,
                        title,
                        chapter_path,
                        context,
                    )?)
                }
                Event::End(Tag::Image(link_type, path, title)) => {
                    //Event::Start(Tag::Image(link_type, imagepathcowstr, title))
                    Event::End(parse_image_tag(
                        link_type,
                        path,
                        title,
                        chapter_path,
                        context,
                    )?)
                }
                _ => event,
            })
        })
        .collect::<std::io::Result<Vec<Event>>>()?;
    let mut new_content = String::new();

    pulldown_cmark_to_cmark::cmark(parser.into_iter(), &mut new_content)
        .expect("Event mod is minimal, must work. qed");
    Ok(new_content)
}

/// Take the values of a Tag::Image and create a new Tag::Image
/// while simplyfying the path and also copying the image file to the target directory
fn parse_image_tag<'a>(
    link_type: LinkType,
    path: CowStr<'a>,
    title: CowStr<'a>,
    _chapter_path: &'a Path,
    context: &'a RenderContext,
) -> std::io::Result<Tag<'a>> {
    // cleaning and converting the path found.
    let imagefn = path.as_ref().strip_prefix("./").unwrap_or(path.as_ref());
    let source = &context.config.book.src.join(imagefn);
    let sourceimage = &context.root.join(&source);
    let targetimage = &context.destination.join(&source);

    if sourceimage != targetimage {
        log::debug!(
            "Copying {} -> {}",
            sourceimage.display(),
            targetimage.display()
        );
        fs::create_dir_all(targetimage.parent().unwrap())?;
        fs::copy(&sourceimage, &targetimage)?;
    }
    // create the new image
    Ok(Tag::Image(link_type, imagefn.to_owned().into(), title))
}
