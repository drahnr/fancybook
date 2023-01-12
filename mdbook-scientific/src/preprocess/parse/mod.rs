use super::*;

pub(crate) mod types;
pub use self::types::*;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitTagPosition<'a> {
    /// Position in line + columns
    lico: LiCo,
    /// Offset in bytes from the beginning of the string
    byte_offset: usize,
    /// start or end
    which: Dollar<'a>,
}

pub fn dollar_split_tags_iter<'a>(source: &'a str) -> impl Iterator<Item = SplitTagPosition<'a>> {
    let mut is_code_block = false;
    let mut is_pre_block = false;
    let mut is_dollar_block = false;
    source
        .lines()
        .enumerate()
        .scan(
            0_usize,
            move |previous_byte_count, (lineno, line_content)| {
                let current_char_cnt = line_content.chars().count();
                // handle block content

                let byte_offset = *previous_byte_count; // byte offset of the start of the line
                *previous_byte_count += line_content.len() + "\n".len(); // update for the next iteration with the current line length plus newline

                let mut current = LiCo { lineno, column: 1 };

                // FIXME NOT OK, could also be further in
                if line_content.starts_with("<pre") {
                    is_pre_block = true;
                    return None;
                }

                if line_content.starts_with("</pre>") {
                    is_pre_block = false;
                    return None;
                }

                if is_pre_block {
                    return None;
                }

                // FIXME use a proper markdown/commonmark parser, it's unfixable this
                // way i.e pre start and end in one line or multiple..
                if line_content.starts_with("```") {
                    is_code_block = !is_code_block;
                }
                if is_code_block {
                    return None;
                }

                if line_content.starts_with("$$") {
                    is_dollar_block = !is_dollar_block;
                    return Some(
                        vec![SplitTagPosition {
                            which: if is_dollar_block {
                                Dollar::Start(&line_content[..("$$".len())])
                            } else {
                                Dollar::End(&line_content[..("$$".len())])
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
                                current.column = il_char_offset;
                                let dollar = SplitTagPosition {
                                    which: if is_between_dollar_content {
                                        Dollar::Start(&line_content[il_byte_offset..][..1])
                                    } else {
                                        Dollar::End(&line_content[il_byte_offset..][..1])
                                    },
                                    lico: current,
                                    byte_offset: byte_offset + il_byte_offset,
                                };
                                return Some(dollar);
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
                            column: current_char_cnt + 1,
                        },
                        byte_offset: line_content.len(),
                        which: Dollar::End(""),
                    })
                }
                Some(tagswpos.into_iter())
            },
        )
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

pub(crate) fn iter_over_dollar_encompassed_blocks<'a>(
    source: &'a str,
    iter: impl Iterator<Item = SplitTagPosition<'a>>,
) -> impl Iterator<Item = Tagged<'a>> {
    // make sure the first part is kept if it doesn't start with a dollar sign
    let mut iter = iter.peekable();
    let pre = match iter.peek() {
        Some(nxt) if nxt.byte_offset > 0 => {
            let byte_range = 0..(nxt.byte_offset);
            let s = &source[byte_range.clone()];
            Some(Tagged::Keep(Content {
                // content without the $ delimiters FIXME
                s,
                start: LiCo {
                    lineno: 1,
                    column: 1,
                },
                end: nxt.lico,
                byte_range,
                start_del: Dollar::Empty,
                end_del: nxt.which,
            }))
        }
        _ => None,
    };
    let iter = iter.tuple_windows().enumerate().map(
        move |(
            idx,
            (
                start @ SplitTagPosition {
                    byte_offset: start_byte_offset,
                    which: start_which,
                    ..
                },
                end @ SplitTagPosition {
                    byte_offset: end_byte_offset,
                    which: end_which,
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
                start_del: start.which,
                end_del: end.which,
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
    pre.into_iter().chain(iter)
}
