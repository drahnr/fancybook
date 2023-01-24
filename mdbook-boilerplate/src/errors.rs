#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SemVer(#[from] semver::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    MdBook(#[from] mdbook::errors::Error),

    #[error("Provided renderer is not supported: {0}")]
    RendererNotSupported(String),

    #[error("Failed to find `{name}`")]
    Which { source: which::Error, name: String },
}
