use std::path::{Path, PathBuf};

use crate::errors::AppError;

pub async fn convert(input: &Path) -> Result<PathBuf, AppError> {
    if !input.exists() {
        return Err(AppError::ConversionFailed("PDF input no longer exists".into()));
    }
    Ok(input.to_path_buf())
}
