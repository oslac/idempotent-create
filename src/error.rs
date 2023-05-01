use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fmt::Formatter;

/// Error type for any internal error.
pub type OpaqueError = color_eyre::Report;

/// Used when sending error response for body.
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
}

/// This will traverse the entire error chain.
pub fn get_error_cause(e: &impl Error, f: &mut Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
