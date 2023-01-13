mod fragments;
mod preprocess;

use crate::errors::Error;
use fs_err as fs;
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

            let fragment_path = cfg
                .get("fragment_path")
                .map(|x| x.as_str().expect("Fragment path is valid UTF8. qed"))
                .unwrap_or("fragments/");

            let fragment_path = Path::new(fragment_path);

            fs::create_dir_all(fragment_path)?;

            let fragment_path = fs::canonicalize(fragment_path)?;
            log::info!("Using fragment path: {}", fragment_path.display());

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
                        &ch.content,
                        &chapter_number,
                        &chapter_name,
                        chapter_path.as_path(),
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

            // the output path is `src/storage/assets`, which get copied to the output directory
            let asset_path = cfg
                .get("assets")
                .map(|x| x.as_str().expect("Assumes valid UTF8 for assets. qed"))
                .unwrap_or("src/");
            let dest = ctx.root.join(asset_path);

            if !dest.exists() {
                fs::create_dir_all(&dest)?;
            }

            let dest = fs::canonicalize(dest)?;

            // copy all used fragments
            if fragment_path != dest {
                for fragment in used_fragments {
                    let from = fragment_path.join(&fragment);
                    let to = dest.join(&fragment);
                    log::info!("Copying {} -> {}", from.display(), to.display());
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