use crate::errors;
use std::path::PathBuf;
use std::str::FromStr;
pub(crate) use crate::preprocess::parse::types::*;

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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "tectonic" => Self::Tectonic,
            "latex" => Self::Latex,
            "markdown" => Self::Markdown,
            "html" => Self::Html,
            s => return Err(errors::Error::RendererNotSupported(s.to_owned())),
        })
    }
}


/// Parsed content reference with a path to the replacement svg
pub struct Replacement<'a> {
    pub content: Content<'a>,

    /// Intermediate representation if there is any, directly usable with latex/tectonic backends;.
    pub(crate) intermediate: Option<String>,
    pub svg: PathBuf,
}

impl<'a> Replacement<'a> {
    pub fn inner_str_or_intermediate(&'a self) -> &'a str {
        if let Some(ref intermediate) = self.intermediate {
            intermediate.as_str()
        } else {
            self.content.trimmed().as_str()
        }
    }
}
