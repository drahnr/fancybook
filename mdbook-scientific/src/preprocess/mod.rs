use fs_err as fs;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::errors::{Result, ScientificError};
use crate::fragments;
use crate::types::*;
use mathyank::iter_over_dollar_encompassed_blocks;
use mathyank::*;

mod format;
pub use self::format::*;

pub fn replace_blocks(
    fragment_path: impl AsRef<Path>,
    asset_path: impl AsRef<Path>,
    source: &str,
    chapter_number: &str,
    chapter_name: &str,
    chapter_path: impl AsRef<Path>,
    renderer: SupportedRenderer,
    used_fragments: &mut Vec<PathBuf>,
    references: &mut ReferenceTracker,
) -> Result<String> {
    let chapter_path = chapter_path.as_ref();
    let fragment_path = fragment_path.as_ref();
    fs::create_dir_all(fragment_path)?;

    let asset_path = asset_path.as_ref();

    let iter = dollar_split_tags_iter(source);
    let s = iter_over_dollar_encompassed_blocks(source, iter)
        .map(|tagged| match tagged {
            Tagged::Keep(content) => Ok(content.as_str().to_owned()),
            Tagged::Replace(content) => {
                let replaced = if content.start_del.is_block() || content.end_del.is_block() {
                    log::debug!("Found block");
                    transform_block_as_needed(
                        &content,
                        fragment_path,
                        asset_path,
                        &chapter_number,
                        &chapter_name,
                        &chapter_path,
                        references,
                        used_fragments,
                        renderer,
                    )
                } else {
                    log::debug!("Found inline");
                    transform_inline_as_needed(
                        &content,
                        fragment_path,
                        asset_path,
                        chapter_number,
                        chapter_name,
                        chapter_path,
                        references,
                        used_fragments,
                        renderer,
                    )
                };
                replaced
            }
        })
        .collect::<Result<Vec<String>>>()?
        .into_iter()
        .collect::<String>();
    Ok(s)
}

fn transform_block_as_needed<'a>(
    content: &Content<'a>,
    fragment_path: impl AsRef<Path>,
    asset_path: impl AsRef<Path>,
    chapter_number: &str,
    chapter_name: &str,
    _chapter_path: impl AsRef<Path>,
    references: &mut ReferenceTracker,
    used_fragments: &mut Vec<PathBuf>,
    renderer: SupportedRenderer,
) -> Result<String> {
    let fragment_path = fragment_path.as_ref();
    let asset_path = asset_path.as_ref();

    let mut figures_counter = 0;
    let mut equations_counter = 0;

    let mut add_object = move |replacement: &mut Replacement<'_>,
                               refer: Option<&str>,
                               title: Option<&str>|
          -> String {
        let fragment_file = replacement.svg_fragment_file.as_path();
        used_fragments.push(fragment_file.to_owned());

        if let Some(title) = title {
            let refer = refer.unwrap_or("unknown var");
            figures_counter += 1;
            references.add(
                refer,
                format!("Figure {}{}", chapter_number, figures_counter),
            );

            format_figure(
                replacement,
                refer,
                &chapter_number,
                figures_counter,
                title,
                renderer,
            )
        } else if let Some(refer) = refer.filter(|s| !s.is_empty()) {
            equations_counter += 1;
            references.add(refer, format!("{}{}", chapter_number, equations_counter));
            format_equation_block(
                replacement,
                refer,
                &chapter_number,
                equations_counter,
                renderer,
            )
        } else {
            format_equation_block(
                replacement,
                "",
                &chapter_number,
                equations_counter,
                renderer,
            )
        }
    };

    let BlockEqu {
        kind, refer, title, ..
    } = BlockEqu::try_from(content)?;

    let mut replacement = match kind {
        EquBlockKind::Latex => fragments::parse_latex(fragment_path, asset_path, &content)?,
        EquBlockKind::GnuPlot => fragments::parse_gnuplot(fragment_path, asset_path, &content)?,
        EquBlockKind::GnuPlotOnly => {
            fragments::parse_gnuplot_only(fragment_path, asset_path, &content)?
        }
        EquBlockKind::Equation => fragments::generate_replacement_file_from_template(
            fragment_path,
            asset_path,
            &content,
            1.6,
            chapter_number,
            chapter_name,
        )?,
    };
    let replacement = add_object(&mut replacement, refer, title);

    let (emoji, desc) = kind.as_emoji_w_desc();
    log::info!("{emoji} Found block {desc}");
    Ok(replacement)
}

/// Transform an inline equation such as
/// ```text
/// Hello sir, I am an equation $a_b(x) = b \times x$ which should render inline!
/// ```
/// Can also be used to reference block equations via `$ref:hello$` where `hello` 'd be the block equation name.
fn transform_inline_as_needed<'a>(
    content: &Content<'a>,
    fragment_path: impl AsRef<Path>,
    asset_path: impl AsRef<Path>,
    chapter_number: &str,
    chapter_name: &str,
    _chapter_path: impl AsRef<Path>,
    references: &mut ReferenceTracker,
    used_fragments: &mut Vec<PathBuf>,
    renderer: SupportedRenderer,
) -> Result<String> {
    let fragment_path = fragment_path.as_ref();
    let asset_path = asset_path.as_ref();
    let lineno = content.start.lineno;

    use mathyank::*;
    match Inline::try_from(content)? {
        Inline::Reference(reference) => {
            let Reference { ref_kind, refere } = reference;
            let title = references
                .get(refere)
                .ok_or(ScientificError::InvalidReference {
                    to: refere.to_owned(),
                    lineno,
                })?;
            let title = title.as_ref();
            let replacement = match ref_kind {
                RefKind::Bibliography => format_bib_reference(refere, title, renderer),
                RefKind::Figure => format_fig_reference(refere, title, renderer),
                RefKind::Equation => format_equ_reference(refere, title, renderer),
            };

            let (emoji, desc) = ref_kind.as_emoji_w_desc();
            log::info!("{emoji} Found reference {desc}");
            Ok(replacement)
        }
        Inline::Equation(_equ) => {
            let replacement = fragments::generate_replacement_file_from_template(
                fragment_path,
                asset_path,
                &content,
                1.3,
                chapter_number,
                chapter_name,
            )?;
            let res = format_equation_inline(&replacement, renderer);
            used_fragments.push(replacement.svg_fragment_file);
            Ok(res)
        }
    }
}
