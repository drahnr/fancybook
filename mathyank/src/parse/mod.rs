use itertools::Itertools;

pub mod types;
pub use self::types::*;

#[cfg(test)]
mod tests;

pub fn dollar_split_tags_iter<'a>(
    source: &'a str,
) -> impl Iterator<Item = SplitTagPosition<'a>> + Clone {
    let mut is_code_block = false;
    let mut is_pre_block = false;
    let mut is_dollar_block = false;
    let mut previous_byte_count = 0;
    source
        .lines()
        .enumerate()
        .filter_map(move |(lineno, line_content)| {
            let current_char_cnt = line_content.chars().count();
            // handle block content

            let byte_offset = previous_byte_count; // byte offset of the start of the line
            previous_byte_count += line_content.len() + "\n".len(); // update for the next iteration with the current line length plus newline

            log::trace!("Processing line {lineno}: \"{line_content}\"");

            let mut current = LiCo { lineno, column: 1 };

            // FIXME NOT OK, could also be further in
            if line_content.starts_with("<pre") {
                is_pre_block = true;
            }

            if line_content.starts_with("</pre>") {
                is_pre_block = false;
                return None;
            }

            if is_pre_block {
                log::debug!("Skipping, active <pre></pre>");
                return None;
            }

            // FIXME use a proper markdown/commonmark parser, it's unfixable this
            // way i.e pre start and end in one line or multiple..
            if line_content.starts_with("```") {
                is_code_block = !is_code_block;
            }
            if is_code_block {
                log::debug!("Skipping, active ``` code block");
                return None;
            }

            if line_content.starts_with("$$") {
                is_dollar_block = !is_dollar_block;
                log::debug!("Found $$/block delimiter");
                return Some(
                    vec![SplitTagPosition {
                        delimiter: if is_dollar_block {
                            Marker::Start(&line_content[..("$$".len())])
                        } else {
                            Marker::End(&line_content[..("$$".len())])
                        },
                        lico: current,
                        byte_offset: byte_offset + 0,
                        // char_offset, // TODO
                    }]
                    .into_iter(),
                );
            }

            let mut is_intra_inline_code = false;
            let mut is_between_dollar_content = false;

            // use to collect ranges
            let mut tagswpos = Vec::from_iter(line_content.char_indices().enumerate().filter_map(
                |(il_char_offset, (il_byte_offset, c))| {
                    match c {
                        '$' if !is_intra_inline_code => {
                            is_between_dollar_content = !is_between_dollar_content;

                            log::debug!("Found $/inline delimiter");
                            current.column = il_char_offset;
                            return Some(SplitTagPosition {
                                delimiter: if is_between_dollar_content {
                                    Marker::Start(&line_content[il_byte_offset..][..1])
                                } else {
                                    Marker::End(&line_content[il_byte_offset..][..1])
                                },
                                lico: current,
                                byte_offset: byte_offset + il_byte_offset,
                            });
                        }
                        '`' => {
                            is_intra_inline_code = !is_intra_inline_code;
                        }
                        _ => {}
                    }
                    None
                },
            ));

            if tagswpos.len() & 0x1 != 0 {
                log::warn!("Inserting $-sign at end of line #{lineno}!");
                tagswpos.push(SplitTagPosition {
                    lico: LiCo {
                        lineno,
                        column: current_char_cnt + 1, // inclusive, but it doesn't exist, so we need one _after_
                    },
                    byte_offset: line_content.len(),
                    delimiter: Marker::End(""),
                })
            }
            Some(tagswpos.into_iter())
        })
        .flatten()
}

#[derive(Debug, Clone)]
pub enum Tagged<'a> {
    Replace(Content<'a>),
    Keep(Content<'a>),
}

impl<'a> Into<Content<'a>> for Tagged<'a> {
    fn into(self) -> Content<'a> {
        match self {
            Self::Replace(c) => c,
            Self::Keep(c) => c,
        }
    }
}

impl<'a> AsRef<Content<'a>> for Tagged<'a> {
    fn as_ref(&self) -> &Content<'a> {
        match self {
            Self::Replace(ref c) => c,
            Self::Keep(ref c) => c,
        }
    }
}

// TODO FIXME refactor
// Should only inject tags, rather than keep content
// and derive the rest from that
pub fn iter_over_dollar_encompassed_blocks<'a, I>(
    source: &'a str,
    iter: I,
) -> impl Iterator<Item = Tagged<'a>>
where
    I: Iterator<Item = SplitTagPosition<'a>> + Clone,
{
    // insert a trailing item if the document does not end with i.e. a `$` sign
    let last = iter
        .clone()
        .last()
        .filter(|tag| tag.byte_offset < source.len())
        .map(|tag| {
            let byte_range = (tag.byte_offset + tag.delimiter.as_str().len())..source.len();
            let s = &source[byte_range.clone()];
            let (last_lineno, last_linecontent) =
                s.lines().enumerate().last().unwrap_or_else(|| (0, source)); // for empty or no newlines, the iterator does not yield anything

            let end = LiCo {
                lineno: last_lineno + 1,
                column: last_linecontent.chars().count(),
            };
            Tagged::Keep(Content {
                s,
                start: tag.lico,
                end,
                byte_range,
                start_del: tag.delimiter,
                end_del: Marker::EndOfDocument(end, source.len()),
            })
        });

    // make sure the first part is kept if it doesn't start with a dollar sign
    let mut iter = iter.peekable();
    let pre = match iter.peek() {
        Some(nxt) if nxt.byte_offset > 0 => {
            let byte_range = 0..(nxt.byte_offset);
            let s = &source[byte_range.clone()];
            let start = LiCo {
                lineno: 1,
                column: 1,
            };
            Some(Tagged::Keep(Content {
                // content including the $ delimiters
                s,
                start,
                end: nxt.lico,
                byte_range,
                start_del: Marker::StartOfDocument(start, 0),
                end_del: nxt.delimiter,
            }))
        }
        Some(_n) => None, // first tag is the very beginning
        None => {
            // empty? No `$` in the input? Make it one big keep
            let start = LiCo {
                lineno: 1,
                column: 1,
            };
            let (last_lineno, last_linecontent) = source
                .lines()
                .enumerate()
                .last()
                .unwrap_or_else(|| (0, source)); // for empty or no newlines, the iterator does not yield anything

            let end = LiCo {
                lineno: last_lineno + 1,
                column: last_linecontent.chars().count().saturating_sub(0),
            };
            Some(Tagged::Keep(Content {
                // content including the $ delimiters
                s: source,
                start,
                end,
                byte_range: 0..(source.len()),
                start_del: Marker::StartOfDocument(start, 0),
                end_del: Marker::EndOfDocument(end, source.len()),
            }))
        } // empty iter shall stay empty
    };
    let iter = iter.tuple_windows().enumerate().map(
        move |(
            idx,
            (
                start @ SplitTagPosition {
                    byte_offset: start_byte_offset,
                    delimiter: start_which,
                    ..
                },
                end @ SplitTagPosition {
                    byte_offset: end_byte_offset,
                    delimiter: end_which,
                    ..
                },
            ),
        )| {
            let replace = idx & 0x1 == 0;
            let byte_range = if replace {
                // replace must _include_ the `$`-signs
                start_byte_offset..(end_byte_offset + end_which.as_ref().chars().count())
            } else {
                // first character might not exist, so this was injected and hence
                // would skip the first character
                let skip_dollar = if start_byte_offset == 0 {
                    0
                } else {
                    {
                        start_which.as_ref().chars().count()
                    }
                };
                (start_byte_offset + skip_dollar)..end_byte_offset
            };

            // not within, so just return a string
            let content = Content {
                // content _including_ the $ delimiters
                s: &source[byte_range.clone()],
                start: start.lico,
                end: end.lico,
                byte_range,
                // delimiters
                start_del: start.delimiter,
                end_del: end.delimiter,
            };

            if replace {
                assert!(
                    content.s.starts_with(content.start_del.as_str()),
                    ">{}< should start with {}",
                    content.s,
                    content.start_del.as_str()
                );
                assert!(
                    content.s.ends_with(content.end_del.as_str()),
                    ">{}< should end with {}",
                    content.s,
                    content.end_del.as_str()
                );
            }
            if replace {
                Tagged::Replace(content)
            } else {
                Tagged::Keep(content)
            }
        },
    );

    pre.into_iter().chain(iter).chain(last.into_iter())
}
