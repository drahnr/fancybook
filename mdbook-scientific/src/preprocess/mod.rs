use fs_err as fs;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::errors::{Error, Result};
use crate::fragments;
use crate::types::*;

mod format;
pub use self::format::*;

pub mod parse;
pub use self::parse::*;


pub fn replace_blocks(
    fragment_path: impl AsRef<Path>,
    source: &str,
    chapter_number: &str,
    chapter_name: &str,
    chapter_path: impl AsRef<Path>,
    renderer: SupportedRenderer,
    used_fragments: &mut Vec<PathBuf>,
    references: &mut HashMap<String, String>,
) -> Result<String> {
    let chapter_path = chapter_path.as_ref();
    let fragment_path = fragment_path.as_ref();
    fs::create_dir_all(fragment_path)?;

    let iter = dollar_split_tags_iter(source);
    let s = iter_over_dollar_encompassed_blocks(source, iter)
        .map(|tagged| match tagged {
            Tagged::Keep(content) => Ok(content.as_str().to_owned()),
            Tagged::Replace(content) => {
                debug_assert_eq!(content.start_del.is_block(), content.end_del.is_block());
                if content.start_del.is_block() {
                    log::debug!("Found block");
                    transform_block_as_needed(
                        &content,
                        fragment_path,
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
                        chapter_number,
                        chapter_name,
                        chapter_path,
                        references,
                        used_fragments,
                        renderer,
                    )
                }
            }
        })
        .collect::<Result<Vec<String>>>()?
        .into_iter()
        .join("\n");
    Ok(s)
}

fn transform_block_as_needed<'a>(
    content: &Content<'a>,
    fragment_path: impl AsRef<Path>,
    chapter_number: &str,
    chapter_name: &str,
    _chapter_path: impl AsRef<Path>,
    references: &mut HashMap<String, String>,
    used_fragments: &mut Vec<PathBuf>,
    renderer: SupportedRenderer,
) -> Result<String> {
    let dollarless = content.trimmed();
    let fragment_path = fragment_path.as_ref();

    let mut figures_counter = 0;
    let mut equations_counter = 0;

    let mut add_object =
        move |replacement: &Replacement<'_>, refer: &str, title: Option<&str>| -> String {
            let file = replacement.svg.as_path();
            used_fragments.push(file.to_owned());

            if let Some(title) = title {
                figures_counter += 1;
                references.insert(
                    refer.to_string(),
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
            } else if !refer.is_empty() {
                equations_counter += 1;
                references.insert(
                    refer.to_string(),
                    format!("{}{}", chapter_number, equations_counter),
                );
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
                    refer,
                    &chapter_number,
                    equations_counter,
                    renderer,
                )
            }
        };

    let params = Vec::from_iter(
        dollarless
            .parameters
            .as_ref()
            .map(|&s| s.splitn(3, ',').map(|s| s.trim()))
            .into_iter()
            .flatten(),
    );

    let (msg, replacement) = match &params[..] {
        ["latex", refer, title] => (
            "🌋 tex",
            fragments::parse_latex(fragment_path, &content)
                .map(|ref file| add_object(file, refer, Some(title))),
        ),
        ["gnuplot", refer, title] => (
            "📈 figure",
            fragments::parse_gnuplot(fragment_path, &content)
                .map(|ref file| add_object(file, refer, Some(title))),
        ),
        ["gnuplotonly", refer, title] => (
            "📈 figure",
            fragments::parse_gnuplot_only(fragment_path, &content)
                .map(|ref file| add_object(file, refer, Some(title))),
        ),

        ["equation", refer] | ["equ", refer] => (
            "🧮 equation",
            fragments::generate_replacement_file_from_template(fragment_path, &content, 1.6, chapter_number, chapter_name)
                .map(|ref file| add_object(file, refer, None)),
        ),

        ["equation"] | ["equ"] | _ => (
            "🧮 equation",
            fragments::generate_replacement_file_from_template(fragment_path, &content, 1.6, chapter_number, chapter_name)
                .map(|ref file| add_object(file, "", None)),
        ),
    };

    let mut iter = msg.chars();
    let emoji = iter.next().unwrap();
    let msg = String::from_iter(iter.skip(1));
    log::info!("{} Found block {}", emoji, msg);
    replacement
}

/// Transform an inline equation such as
/// ```text
/// Hello sir, I am an equation $a_b(x) = b \times x$ which should render inline!
/// ```
/// Can also be used to reference block equations via `$ref:hello$` where `hello` 'd be the block equation name.
fn transform_inline_as_needed<'a>(
    content: &Content<'a>,
    fragment_path: impl AsRef<Path>,
    chapter_number: &str,
    chapter_name: &str,
    _chapter_path: impl AsRef<Path>,
    references: &HashMap<String, String>,
    used_fragments: &mut Vec<PathBuf>,
    renderer: SupportedRenderer,
) -> Result<String> {
    let dollarless = content.trimmed();
    let fragment_path = fragment_path.as_ref();
    let lineno = content.start.lineno;

    if let Some(stripped) = dollarless.strip_prefix("ref:") {
        let elms = stripped.split(':').collect::<Vec<&str>>();
        let (msg, replacement) = match &elms[..] {
            ["fig", refere] => (
                "📈 figure",
                references
                    .get::<str>(refere)
                    .ok_or(Error::InvalidReference {
                        to: elms[1].to_owned(),
                        lineno,
                    })
                    .map(|x| format!(r#"<a class="fig_ref" href='#{}'>{}</a>"#, elms[1], x)),
            ),
            ["bib", refere] => (
                "📚 bibliography",
                references
                    .get::<str>(refere)
                    .ok_or(Error::InvalidReference {
                        to: elms[1].to_owned(),
                        lineno,
                    })
                    .map(|x| {
                        format!(
                            r#"<a class="bib_ref" href='bibliography.html#{}'>{}</a>"#,
                            elms[1], x
                        )
                    }),
            ),
            ["equ", refere] => (
                "🧮 equation",
                references
                    .get::<str>(refere)
                    .ok_or(Error::InvalidReference {
                        to: elms[1].to_owned(),
                        lineno,
                    })
                    .map(|x| format!(r#"<a class="equ_ref" href='#{}'>Eq. ({})</a>"#, elms[1], x)),
            ),
            [kind, _] => {
                return Err(Error::UnknownReferenceKind {
                    kind: kind.to_owned().to_owned(),
                    lineno,
                })
            }
            _ => {
                return Err(Error::UnexpectedReferenceArgCount {
                    count: elms.len(),
                    lineno,
                })
            }
        };
        let mut iter = msg.chars();
        let emoji = iter.next().unwrap();
        let msg = String::from_iter(iter.skip(1));
        log::info!("{} Found inline {}", emoji, msg);
        replacement
    } else {
        fragments::generate_replacement_file_from_template(fragment_path, &content, 1.3, chapter_number, chapter_name).map(
            |replacement| {
                let res = format_equation_inline(&replacement, renderer);
                used_fragments.push(replacement.svg);
                res
            },
        )
    }
}
