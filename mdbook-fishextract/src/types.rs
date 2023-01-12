use crate::errors;
use sha2::Digest;
use sha2::Sha256;
use std::str::FromStr;

/// Short hash.
pub fn short_hash(input: impl AsRef<str>) -> String {
    let mut sh = Sha256::new();
    sh.update(input.as_ref().as_bytes());
    let mut out = format!("{:x}", sh.finalize());
    out.truncate(10);
    out
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
