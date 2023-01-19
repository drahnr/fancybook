use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown reference type {ref_kind}")]
    UnknownRefKind { ref_kind: String },
    #[error("Doesn't have a `ref` prefix: {s}")]
    MissingRefPrefix { s: String },
    #[error("Missing ref kind: {s}")]
    MissingRefKind { s: String },
    #[error("Missing identifier: {s}")]
    MissingIdentifier { s: String },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum RefKind {
    Equation,
    // inline equations cannot be referenced
    Figure,
    Bibliography,
}

impl RefKind {
    pub fn as_emoji_w_desc(&self) -> (&'static str, &'static str) {
        match self {
            Self::Figure => ("📈", "figure"),
            Self::Equation => ("🧮", "equation"),
            Self::Bibliography => ("📚", "bibliography"),
        }
    }

    pub fn as_emoji(&self) -> &'static str {
        self.as_emoji_w_desc().0
    }

    pub fn as_desc(&self) -> &'static str {
        self.as_emoji_w_desc().1
    }
}
impl FromStr for RefKind {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "equ" | "equation" => Self::Equation,
            "fig" | "figure" => Self::Figure,
            "bib" | "bibliography" => Self::Bibliography,
            abbrev => {
                return Err(Self::Err::UnknownRefKind {
                    ref_kind: abbrev.to_owned(),
                })
            }
        })
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Reference<'a> {
    /// The specified ID to be used as an identifier for lookups
    pub refere: &'a str,
    /// The type of reference, _what_ type it refers to
    pub ref_kind: RefKind,
}

impl<'a> Reference<'a> {
    fn from_str<'b>(s: &'b str) -> Result<Self, Error> where 'b:'a {
        let mut elms = s.split(':');
        let ref_kind = if let Some(first) = elms.next() {
            RefKind::from_str(first)?
        } else {
            return Err(Error::MissingRefKind { s: s.to_owned() });
        };
        let refere = elms
            .next()
            .ok_or_else(|| Error::MissingIdentifier { s: s.to_owned() })?;
        Ok(Self {
            refere,
            ref_kind,
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Item<'a> {
    /// An inline block, could be a reference to an equation or figure, or an equation itself
    Inline(Inline<'a>),
    /// A block equation or figure, could hold an additional refer id and title
    Block(BlockEqu<'a>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Inline<'a> {
    /// An inline equation that's actually just referencing
    Reference(Reference<'a>),
    /// A true inline equation
    Equation(InlineEqu<'a>),
}

impl<'a, 'b> TryFrom<&'b Content<'a>> for Inline<'a> where 'b:'a {
    type Error = Error;
    fn try_from(content: &'b Content<'a>) -> Result<Self, Error> {
        let trimmed = content.trimmed();
        Ok(if let Some(stripped) = trimmed.strip_prefix("ref:") {
            Self::Reference(Reference::from_str(stripped)?)
        } else {
            Self::Equation(InlineEqu::from(content))
        })
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InlineEqu<'a> {
    pub content: &'a Content<'a>,
}

impl<'a> From<&'a Content<'a>> for InlineEqu<'a>
{
    fn from(value: &'a Content<'a>) -> Self {
        Self { content: value }
    }
}
/// The type of block
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EquBlockKind {
    Latex,
    GnuPlot,
    GnuPlotOnly,
    Equation,
}

impl FromStr for EquBlockKind {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "gnuplot" => Self::GnuPlot,
            "gnuplotonly" => Self::GnuPlotOnly,
            "equ" | "equation" => Self::Equation,
            abbrev => {
                return Err(Self::Err::UnknownRefKind {
                    ref_kind: abbrev.to_owned(),
                })
            }
        })
    }
}

impl EquBlockKind {
    pub fn as_emoji_w_desc(&self) -> (&'static str, &'static str) {
        match self {
            Self::Latex => ("🌋", "tex"),
            Self::Equation => ("🧮", "equation"),
            Self::GnuPlot | Self::GnuPlotOnly => ("📈", "figure"),
        }
    }

    pub fn as_emoji(&self) -> &'static str {
        self.as_emoji_w_desc().0
    }

    pub fn as_desc(&self) -> &'static str {
        self.as_emoji_w_desc().1
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BlockEqu<'a> {
    pub content: &'a Content<'a>,
    pub kind: EquBlockKind,
    pub refer: Option<&'a str>,
    pub title: Option<&'a str>,
}

impl<'a, 'b> TryFrom<&'b Content<'a>> for BlockEqu<'a>
where
    'b: 'a,
{
    type Error = Error;
    fn try_from(content: &'b Content<'a>) -> Result<Self, Error> {
        let mut params = content
            .trimmed()
            .parameters
            .as_ref()
            .map(|&s| s.splitn(3, ',').map(|s| s.trim()))
            .into_iter()
            .flatten();

        let kind = params
            .next()
            .map(|kind_str| EquBlockKind::from_str(kind_str))
            .unwrap_or(Ok(EquBlockKind::Equation))?;
        Ok(Self {
            content,
            kind,
            refer: params.next(),
            title: params.next(),
        })
    }
}

impl<'a, 'b> TryFrom<&'b Content<'a>> for Item<'a>
where
    'b: 'a,
{
    type Error = Error;
    fn try_from(content: &'b Content) -> Result<Self, Error> {
        Ok(
            if content.start_del.is_block() || content.end_del.is_block() {
                Self::Block(BlockEqu::try_from(content)?)
            } else {
                Self::Inline(Inline::try_from(content)?)
            },
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiCo {
    /// Base 1 line number
    pub lineno: usize,
    /// Base 1 column number
    pub column: usize,
}

/// A dollar sign or maybe two, or three.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitTagPosition<'a> {
    /// Position in line + columns
    pub lico: LiCo,
    /// Offset in bytes from the beginning of the string
    pub byte_offset: usize,
    /// start or end, or start/end of document
    pub delimiter: Marker<'a>,
}

/// A content reference
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

/// Removes the delimiters and possible parameters
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
        // FIXME split functionality for finding start and end, and start of doc and end of doc
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

                debug_assert_eq!(dbg!(&content.as_str()[..(DELIM.len())]), dbg!(DELIM));

                Trimmed {
                    trimmed: &content.s[start.1..end.1],
                    parameters: None,
                    start: start.0,
                    end: end.0,
                    byte_range: start.1..end.1,
                }
            }
            // FIXME incorrect, StartOfDocument and EndOfDocument are "" as well
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
