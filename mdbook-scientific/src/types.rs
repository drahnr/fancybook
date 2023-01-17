use crate::errors;
pub(crate) use crate::parse::types::*;
use std::path::PathBuf;
use std::str::FromStr;

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
#[derive(Debug)]
pub struct Replacement<'a> {
    pub content: Content<'a>,

    /// Intermediate representation if there is any, directly usable with latex/tectonic backends;.
    pub(crate) intermediate: Option<String>,
    /// Path to the rendered context, possibly an svg or pdf or png file. Double check.
    pub svg_fragment_file: PathBuf,
    /// Path where the file will ulitmately reside in, must be used for referencing in output
    pub svg_asset_file: PathBuf,
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
