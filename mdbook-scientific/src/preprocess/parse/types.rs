/// A dollar sign or maybe two, or three.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker<'a> {
    Start(&'a str),
    End(&'a str),
    EndOfDocument,
    StartOfDocument,
}

impl<'a> Marker<'a> {
    pub fn is_block(&self) -> bool {
        self.as_ref().starts_with("$$")
    }

    pub fn as_str(&self) -> &'a str {
        match self {
            Self::Start(s) => s,
            Self::End(s) => s,
            Self::EndOfDocument | Self::StartOfDocument => "",
        }
    }
}

impl<'a> AsRef<str> for Marker<'a> {
    fn as_ref(&self) -> &'a str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiCo {
    /// Base 1 line number
    pub lineno: usize,
    /// Base 1 column number
    pub column: usize,
}

/// A content reference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content<'a> {
    /// Content between `start` and `end` including.
    pub s: &'a str,
    /// From (including!)
    pub start: LiCo,
    /// Until (including!)
    pub end: LiCo,
    /// Byte range that can be used with the original to extract `s`
    pub byte_range: std::ops::Range<usize>,

    pub start_del: Marker<'a>,
    pub end_del: Marker<'a>,
}

impl<'a> Content<'a> {
    /// Strips dollars and any prefix signs
    pub fn trimmed(&self) -> Trimmed<'a> {
        Trimmed::from(self)
    }

    pub fn as_str(&self) -> &'a str {
        self.s
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trimmed<'a> {
    /// Content between `start` and `end`, _excluding_ start and end, without any delimiters.
    pub trimmed: &'a str,
    /// The parameters to be parsed
    pub parameters: Option<&'a str>,
    /// From (including!)
    pub start: LiCo,
    /// Until (including!)
    pub end: LiCo,
    /// Byte range that can be used with the original to extract `s`
    pub byte_range: std::ops::Range<usize>,
}

fn annotate(s: &str) -> Vec<(LiCo, usize, char)> {
    s.char_indices()
        .scan(
            LiCo {
                lineno: 1,
                column: 0,
            },
            |cursor, (byte_offset, c)| {
                if c == '\n' {
                    cursor.lineno += 1;
                    cursor.column = 0;
                }
                cursor.column += 1;

                Some((cursor.clone(), byte_offset, c))
            },
        )
        .collect()
}

impl<'a, 'b> From<&'b Content<'a>> for Trimmed<'a>
where
    'a: 'b,
{
    fn from(content: &'b Content<'a>) -> Self {
        debug_assert_eq!(content.start_del.as_str(), content.end_del.as_str());

        let dollarless = match content.start_del.as_str() {
            "$$" => {
                const DELIM: &str = "$$";
                let start = content.start;
                let end = content.end;
                assert!(start <= end);

                let v: Vec<_> = annotate(content.s);

                let start = v.iter().find(|&&(_, _, c)| c == '\n').cloned().unwrap();
                // in case there is only one newline enclosed between `$$\n$$`, use the start newline
                let mut iter = v.iter();
                // we need the byte offset after, but the LiCo to be the one before, since it's inclusive
                let end = if let Some(one_after) = iter.rfind(|&&(_, _, c)| c == '\n') {
                    let mut end = iter
                        .next_back()
                        .cloned()
                        .unwrap_or_else(|| one_after.clone());
                    end.1 = one_after.1;
                    if end < start {
                        start
                    } else {
                        end
                    }
                } else {
                    start.clone()
                };

                let first_line = &content.s[..start.1];
                assert_eq!(&first_line[..(DELIM.len())], DELIM);
                assert!(start.1 >= DELIM.len());
                let params = &content.s[(DELIM.len())..start.1];
                let parameters = Some(params).filter(|s| !s.is_empty());

                Trimmed {
                    trimmed: &content.s[start.1..end.1],
                    parameters,
                    start: start.0,
                    end: end.0,
                    byte_range: start.1..end.1,
                }
            }
            "$" => {
                const DELIM: &str = "$";
                let start = content.start;
                let end = content.end;
                assert!(start <= end);

                let v: Vec<_> = annotate(content.s);
                let iter = v.iter();
                let mut iter = iter.skip(DELIM.len());
                let start = iter.next().cloned().unwrap();
                let iter = iter.rev().cloned();
                let last = v.last().cloned().unwrap_or_else(|| start.clone());
                let second_to_last = iter.skip(1).next().unwrap_or_else(|| last.clone());
                let end = (second_to_last.0, last.1);
                // FIXME currently end is _excluding_ but it really should be including

                debug_assert_eq!(dbg!(&content.as_str()[..(DELIM.len())]), dbg!(DELIM));

                Trimmed {
                    trimmed: &content.s[start.1..end.1],
                    parameters: None,
                    start: start.0,
                    end: end.0,
                    byte_range: start.1..end.1,
                }
            }
            // incorrect, StartOfDocument and EndOfDocument are "" as well
            other => unreachable!(
                r#"Only $ or $$ are valid delimiters and only those make it up until here, but found "{other}". qed"#
            ),
        };
        dollarless
    }
}

impl<'a> Trimmed<'a> {
    pub fn as_str(&self) -> &'a str {
        self.trimmed
    }
}

impl<'a> AsRef<str> for Trimmed<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> std::ops::Deref for Trimmed<'a> {
    type Target = &'a str;
    fn deref(&self) -> &Self::Target {
        &self.trimmed
    }
}
