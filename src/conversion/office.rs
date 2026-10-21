use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;
use uuid::Uuid;

use crate::{config::Config, errors::AppError};

pub async fn convert(input: &Path, cfg: &Config) -> Result<PathBuf, AppError> {
    let profile_path = Path::new(&cfg.temp_dir).join(format!("lo-profile-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&profile_path).await?;
    let profile_uri = format!("file://{}", profile_path.display());
    let output_directory = input
        .parent()
        .ok_or_else(|| AppError::ConversionFailed("Input has no parent directory".into()))?;
    let output = tokio::time::timeout(
        Duration::from_secs(cfg.conversion_timeout_secs),
        Command::new(&cfg.libreoffice_path)
            .args([
                "--headless",
                "--norestore",
                "--nologo",
                "--nolockcheck",
                &format!("-env:UserInstallation={profile_uri}"),
                "--convert-to",
                "pdf",
                "--outdir",
                output_directory.to_str().unwrap_or_default(),
                input.to_str().unwrap_or_default(),
            ])
            .output(),
    )
    .await;
    let _ = tokio::fs::remove_dir_all(&profile_path).await;

    match output {
        Err(_) => Err(AppError::ConversionFailed("LibreOffice timed out".into())),
        Ok(Err(error)) => Err(AppError::Io(error)),
        Ok(Ok(output)) if !output.status.success() => Err(AppError::ConversionFailed(
            format!(
                "LibreOffice exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        )),
        Ok(Ok(_)) => {
            let output_path = input.with_extension("pdf");
            if output_path.exists() {
                Ok(output_path)
            } else {
                Err(AppError::ConversionFailed(
                    "LibreOffice produced no PDF output".into(),
                ))
            }
        }
    }
}
