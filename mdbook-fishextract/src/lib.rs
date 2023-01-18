mod mermaid;

use crate::errors::Error;
use fs_err as fs;
use mdbook::book::SectionNumber;
use mermaid::replace_mermaid_charts;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub mod errors;
pub use self::errors::*;

pub mod types;
pub use self::types::*;

#[cfg(test)]
mod tests;

pub struct Fishextract;

impl Fishextract {
    pub fn new() -> Fishextract {
        Fishextract
    }
}

impl Preprocessor for Fishextract {
    fn name(&self) -> &str {
        "fishextract"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        SupportedRenderer::from_str(renderer).is_ok()
    }

    fn run(
        &self,
        ctx: &PreprocessorContext,
        book: Book,
    ) -> std::result::Result<Book, mdbook::errors::Error> {
        self.run_inner(ctx, book)
            .map_err(mdbook::errors::Error::new)
    }
}

fn get_config_value(cfg: &toml::value::Table, key: &str, default: impl Into<PathBuf>) -> PathBuf {
    cfg.get(key)
        .map(|x| x.as_str().expect("Config path is valid UTF8. qed"))
        .map(PathBuf::from)
        .unwrap_or(default.into())
}

fn fragment_path(cfg: &toml::value::Table) -> PathBuf {
    get_config_value(cfg, "fragment_path", "fragments")
}

fn asset_path(cfg: &toml::value::Table) -> PathBuf {
    get_config_value(cfg, "assets", PathBuf::from("src").join("assets"))
}

impl Fishextract {
    fn run_inner(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        if let Some(cfg) = ctx.config.get_preprocessor(self.name()) {
            let renderer = SupportedRenderer::from_str(ctx.renderer.as_str())?;

            let fragment_path = fragment_path(cfg);
            fs::create_dir_all(&fragment_path)?;

            let fragment_path = fs::canonicalize(fragment_path)?;
            log::info!("Using fragment path: {}", fragment_path.display());

            // track all generated assets in a temporary dir
            let mut used_fragments = Vec::new();

            // error acc, only prints the first encountered error
            let mut error = Ok::<_, Error>(());

            // replace mermaid charts with prerendered svgs
            book.for_each_mut(|item| {
                if let BookItem::Chapter(ref mut ch) = item {
                    log::info!(
                        "Processing chapter {} - {}",
                        ch.number
                            .as_ref()
                            .map(|sn| sn.to_string())
                            .unwrap_or("Pre".to_owned()),
                        ch.name.as_str()
                    );
                    log::debug!(
                        "Chapter resides at {}",
                        ch.path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| { "?".to_owned() })
                    );

                    let chapter_number = ch
                        .number
                        .as_ref()
                        .unwrap_or(&SectionNumber::default())
                        .to_string();

                    match replace_mermaid_charts(
                        ch.content.as_str(),
                        &chapter_number,
                        &ch.name,
                        ch.path.as_ref().unwrap_or(&PathBuf::new()),
                        &fragment_path,
                        renderer,
                        &mut used_fragments,
                    ) {
                        Ok(replace_with) => {
                            ch.content = replace_with;
                        }
                        err if error.is_ok() => {
                            error = err.map(|_| ()).map_err(Error::from);
                        }
                        _ => {}
                    };
                }
            });

            error?;

            // copy fragments over to the asset path
            let asset_path = asset_path(cfg);

            fs::create_dir_all(&asset_path)?;

            // copy all used fragments
            if fragment_path != asset_path {
                for fragment in used_fragments {
                    let from = fragment_path.join(&fragment);
                    let to = asset_path.join(&fragment);
                    log::info!(
                        "Copying fishextract to assets dir: {} -> {}",
                        from.display(),
                        to.display()
                    );
                    fs::copy(from, to)?;
                }
            } else {
                log::debug!(
                    "Fragments already in the right place, copying nothing {}",
                    fragment_path.display()
                )
            }

            Ok(book)
        } else {
            Err(Error::KeySectionNotFound)
        }
    }
}
