use crate::errors;
pub(crate) use mathyank::types::*;
use std::path::PathBuf;
use std::str::FromStr;

use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReferenceTracker(HashMap<String, String>);

impl ReferenceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, key: impl AsRef<str>, title: impl AsRef<str>) {
        let key = key.as_ref();
        log::warn!("Addind reference `{}`", key);
        self.0.insert(key.to_string(), title.as_ref().to_string());
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<String> {
        let key = key.as_ref();
        let maybe_value = self.0.get(key);
        log::warn!("Lookup of reference `{}` yieled `{:?}`", key, &maybe_value);
        maybe_value.cloned()
    }
}

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
    type Err = errors::ScientificError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "tectonic" => Self::Tectonic,
            "latex" => Self::Latex,
            "markdown" => Self::Markdown,
            "html" => Self::Html,
            s => return Err(errors::ScientificError::RendererNotSupported(s.to_owned())),
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
