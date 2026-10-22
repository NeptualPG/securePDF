use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;

use crate::{config::Config, errors::AppError};

pub async fn convert(input: &Path, cfg: &Config) -> Result<PathBuf, AppError> {
    let output = input.with_extension("pdf");
    let result = tokio::time::timeout(
        Duration::from_secs(cfg.conversion_timeout_secs),
        Command::new(&cfg.imagemagick_path)
            .args([
                "--",
                input.to_str().unwrap_or_default(),
                output.to_str().unwrap_or_default(),
            ])
            .output(),
    )
    .await;

    match result {
        Err(_) => Err(AppError::ConversionTimeout),
        Ok(Err(error)) => Err(AppError::Io(error)),
        Ok(Ok(output_result)) if !output_result.status.success() => {
            Err(AppError::ConversionFailed(format!(
                "ImageMagick exited with {}: {}",
                output_result.status,
                String::from_utf8_lossy(&output_result.stderr).trim()
            )))
        }
        Ok(Ok(_)) if !output.exists() => Err(AppError::ConversionFailed(
            "ImageMagick produced no PDF output".into(),
        )),
        Ok(Ok(_)) => Ok(output),
    }
}
