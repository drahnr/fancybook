pub use clap::Parser;
pub use color_eyre::eyre::Report;
pub use color_eyre::eyre::Result;
pub use mdbook::preprocess::CmdPreprocessor;
pub use mdbook::preprocess::Preprocessor;
use sha2::Digest;
use sha2::Sha256;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
pub use toml::value::Table;

mod errors;
pub use crate::errors::*;

/// Get a config value
pub fn get_config_value(
    cfg: &toml::value::Table,
    key: &str,
    default: impl Into<PathBuf>,
) -> PathBuf {
    cfg.get(key)
        .map(|x| x.as_str().expect("Config path is valid UTF8. qed"))
        .map(PathBuf::from)
        .unwrap_or(default.into())
}

/// Extract a temporary fragments path
pub fn fragment_path(cfg: &toml::value::Table) -> PathBuf {
    get_config_value(cfg, "fragment_path", "fragments")
}

/// Determine the final assets path
pub fn asset_path(cfg: &toml::value::Table) -> PathBuf {
    get_config_value(cfg, "assets", PathBuf::from("src").join("assets"))
}

/// Short hash. Useful in conjunction with chapter info.
pub fn short_hash(input: impl AsRef<str>) -> String {
    let mut sh = Sha256::new();
    sh.update(input.as_ref().as_bytes());
    let mut out = format!("{:x}", sh.finalize());
    out.truncate(10);
    out
}

/// Enum covering all supported renderers
///
/// Typesafety first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedRenderer {
    Tectonic,
    Latex,
    Markdown,
    Html,
}

impl FromStr for SupportedRenderer {
    type Err = errors::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "tectonic" => Self::Tectonic,
            "latex" => Self::Latex,
            "markdown" => Self::Markdown,
            "html" => Self::Html,
            s => return Err(errors::Error::RendererNotSupported(s.to_owned())),
        })
    }
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[command(subcommand)]
    pub supports: Option<Sub>,
}

#[derive(clap::Subcommand, Debug)]
#[doc = "Check whether a renderer is supported by this preprocessor"]
pub enum Sub {
    Supports { renderer: String },
}

pub fn launch<Pre: Preprocessor + 'static>(
    preprocessor: Pre,
    args: impl Into<Args>,
    prefix: &'static str,
) -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let name = preprocessor.name().to_owned();
    use env_logger::Builder;
    use log::LevelFilter;
    let mut builder = Builder::from_default_env();
    builder.format(move |formatter, record| {
        let name = name.as_str();
        let time = formatter.timestamp();
        let lvl = formatter.default_styled_level(record.level());
        let args = record.args();

        let style = formatter
            .style()
            .set_color(env_logger::fmt::Color::Black)
            .set_intense(true)
            .clone();
        let open = style.value("[");
        let close = style.value("]");
        writeln!(
            formatter,
            "{open}{time} {lvl:5} {prefix} {name} {close} {args}"
        )
    });
    builder.filter(None, LevelFilter::Debug).init();

    log::debug!(
        "{} called from {}!",
        preprocessor.name(),
        std::env::current_dir().unwrap().display()
    );

    let args = args.into();
    if let Some(Sub::Supports { ref renderer }) = args.supports {
        handle_supports(&preprocessor, renderer);
    } else {
        handle_preprocessing(&preprocessor)?;
    }
    Ok(())
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<()> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin()).map_err(Error::MdBook)?;

    let compiled_against = semver::VersionReq::parse(mdbook::MDBOOK_VERSION)?;
    let running_against = semver::Version::parse(ctx.mdbook_version.as_str())?;
    if !compiled_against.matches(&running_against) {
        log::warn!(
            "The {} plugin was built against version {} of mdbook, \
            but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book).map_err(Error::MdBook)?;

    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, renderer: impl AsRef<str>) -> ! {
    let supported = pre.supports_renderer(renderer.as_ref());

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn clap_supports() {
        assert_matches!(Args::try_parse_from(vec!["mdbook-foo", "supports"]), Err(_));
        assert_matches!(Args::try_parse_from(vec!["mdbook-foo", "supports", "tectonic"]).unwrap(),
        Args {
            supports: Some(Sub::Supports{ renderer }),
            ..
        } => {
            assert_eq!(renderer, "tectonic");
        });
    }

    #[test]
    fn clap_supports_no_sub() {
        assert_matches!(Args::try_parse_from(vec!["mdbook-foo"]).unwrap(),
        Args {
            supports: None,
            ..
        } => {
        });
    }
}
