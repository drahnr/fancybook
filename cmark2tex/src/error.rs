#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Svg(#[from] resvg::usvg::Error),

    #[error(transparent)]
    PngEnc(#[from] png::EncodingError),

    #[error("Missing argument, must provide input + output")]
    MissingArg,

    #[error(transparent)]
    Regex(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
