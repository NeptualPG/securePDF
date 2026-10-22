use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;

use crate::{config::Config, errors::AppError};

pub async fn convert(
    input: &Path,
    target: &str,
    page: u32,
    dpi: u32,
    cfg: &Config,
) -> Result<PathBuf, AppError> {
    let prefix = input.with_extension("");
    let output = prefix.with_file_name(format!(
        "{}-{}.{}",
        prefix
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("page"),
        page,
        target
    ));
    let format_flag = if target == "jpg" { "-jpeg" } else { "-png" };
    let result = tokio::time::timeout(
        Duration::from_secs(cfg.conversion_timeout_secs),
        Command::new(&cfg.pdftoppm_path)
            .args([
                format_flag,
                "-r",
                &dpi.to_string(),
                "-f",
                &page.to_string(),
                "-l",
                &page.to_string(),
                input.to_str().unwrap_or_default(),
                prefix.to_str().unwrap_or_default(),
            ])
            .output(),
    )
    .await;

    match result {
        Err(_) => Err(AppError::ConversionTimeout),
        Ok(Err(error)) => Err(AppError::Io(error)),
        Ok(Ok(output_result)) if !output_result.status.success() => {
            Err(AppError::ConversionFailed(format!(
                "pdftoppm exited with {}: {}",
                output_result.status,
                String::from_utf8_lossy(&output_result.stderr).trim()
            )))
        }
        Ok(Ok(_)) if !output.exists() => Err(AppError::ConversionFailed(
            "pdftoppm produced no image output".into(),
        )),
        Ok(Ok(_)) => Ok(output),
    }
}
