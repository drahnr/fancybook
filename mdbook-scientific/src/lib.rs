mod fragments;
mod preprocess;

pub mod parse;
pub use self::parse::*;

use crate::errors::Error;
use fs_err as fs;
use mdbook_boilerplate::{asset_path, fragment_path};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use nom_bibtex::*;

use crate::preprocess::replace_blocks;

pub mod errors;
pub use self::errors::*;

pub mod types;
pub use self::types::*;

#[cfg(test)]
mod tests;

pub struct Scientific;

impl Scientific {
    pub fn new() -> Scientific {
        Scientific
    }
}

impl Preprocessor for Scientific {
    fn name(&self) -> &str {
        "scientific"
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

impl Scientific {
    fn run_inner(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        if let Some(cfg) = ctx.config.get_preprocessor(self.name()) {
            let renderer = SupportedRenderer::from_str(ctx.renderer.as_str())?;

            let fragment_path = fragment_path(&cfg);
            log::info!("Using fragment path: {}", fragment_path.display());

            fs::create_dir_all(&fragment_path)?;
            let fragment_path = fs::canonicalize(fragment_path)?;

            // the output path is `src/assets`, which get copied to the output directory
            let asset_path = asset_path(&cfg);

            log::info!("Using asset path: {}", asset_path.display());
            fs::create_dir_all(&asset_path)?;

            // track which fragments we use to copy them into the assets folder
            let mut used_fragments = Vec::new();
            // track which references are created
            let mut references = HashMap::new();
            // if there occurs an error skip everything and return the error
            let mut error = Ok::<_, Error>(());

            match renderer {
                SupportedRenderer::Markdown | SupportedRenderer::Html => {
                    // load all references in the bibliography and export to html
                    if let (Some(bib), Some(bib2xhtml)) =
                        (cfg.get("bibliography"), cfg.get("bib2xhtml"))
                    {
                        let bib = bib.as_str().unwrap();
                        let bib2xhtml = bib2xhtml.as_str().expect("bib string is valid UTF8. qed");

                        if !Path::new(bib).exists() {
                            return Err(Error::BibliographyMissing(bib.to_owned()));
                        }

                        // read entries in bibtex file
                        let bibtex = fs::read_to_string(bib)?;
                        let bibtex = Bibtex::parse(&bibtex)?;
                        for (i, entry) in bibtex.bibliographies().iter().enumerate() {
                            references
                                .insert(entry.citation_key().to_string(), format!("[{}]", i + 1));
                        }

                        // create bibliography
                        let content = fragments::bib_to_html(bib, bib2xhtml)?;

                        // add final chapter for bibliography
                        let bib_chapter = Chapter::new(
                            "Bibliography",
                            format!("# Bibliography\n{}", content),
                            PathBuf::from("bibliography.md"),
                            Vec::new(),
                        );
                        book.push_item(bib_chapter);
                    }
                }
                SupportedRenderer::Tectonic | SupportedRenderer::Latex => {
                    //native support for bibtex, no need to fuck around
                }
            }

            // process blocks like `$$ .. $$`
            book.for_each_mut(|item| {
                if let Err(_) = error {
                    log::debug!("Previous error, skipping chapter processing.");
                    return;
                }

                if let BookItem::Chapter(ref mut ch) = item {
                    let chapter_number = ch
                        .number
                        .as_ref()
                        .map(|x| x.to_string())
                        .unwrap_or_default();
                    let chapter_path = ch.path.as_ref().cloned().unwrap_or_else(|| PathBuf::new());
                    let chapter_name = ch.name.clone();

                    match replace_blocks(
                        &fragment_path,
                        &asset_path,
                        &ch.content,
                        &chapter_number,
                        &chapter_name,
                        &chapter_path,
                        renderer,
                        &mut used_fragments,
                        &mut references,
                    ) {
                        Ok(mut reconstructed) => {
                            reconstructed.push('\n');
                            if reconstructed != ch.content {
                                // for line in ch.content.lines() {
                                //     eprintln!("- {}", line);
                                // }
                                // for line in reconstructed.lines() {
                                //     eprintln!("+ {}", line);
                                // }
                                ch.content = reconstructed;
                            }
                        }
                        Err(err) => error = Err(err),
                    }
                }
            });

            error?;

            // copy all used fragments
            if fragment_path != asset_path {
                // svg_path is unfortunately the `fragment_path` plus `file` which is an abs path.
                for fragment in used_fragments {
                    let from = fragment;
                    let fragment = from
                        .strip_prefix(&fragment_path)
                        .expect("We feed in a path prefixed, so outcome must maintain that. qed")
                        .to_owned();
                    // let from = fragment_path.join(&fragment);
                    let to = asset_path.join(&fragment);
                    log::info!(
                        "Copying fragment to assets dir: {} -> {}",
                        from.display(),
                        to.display()
                    );
                    fs::copy(from, to)?;
                }
            } else {
                log::debug!(
                    "Fragments already in the right place {}, copying nothing.",
                    asset_path.display()
                )
            }
            Ok(book)
        } else {
            Err(Error::KeySectionNotFound)
        }
    }
}
