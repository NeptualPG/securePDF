use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::process::Command;
use uuid::Uuid;

use crate::{config::Config, errors::AppError};

pub async fn convert(input: &Path, target: &str, cfg: &Config) -> Result<PathBuf, AppError> {
    let profile_path = Path::new(&cfg.temp_dir).join(format!("lo-profile-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&profile_path).await?;
    let profile_uri = format!("file://{}", profile_path.display());
    let output_directory = input
        .parent()
        .ok_or_else(|| AppError::ConversionFailed("Input has no parent directory".into()))?;
    let filter = office_filter(target).ok_or_else(|| {
        AppError::ConversionFailed(format!("Unsupported Office target format: {target}"))
    })?;
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
                filter,
                "--outdir",
                output_directory.to_str().unwrap_or_default(),
                input.to_str().unwrap_or_default(),
            ])
            .output(),
    )
    .await;
    let _ = tokio::fs::remove_dir_all(&profile_path).await;

    match output {
        Err(_) => Err(AppError::ConversionTimeout),
        Ok(Err(error)) => Err(AppError::Io(error)),
        Ok(Ok(output)) if !output.status.success() => Err(AppError::ConversionFailed(format!(
            "LibreOffice exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
        Ok(Ok(_)) => {
            let output_path = input.with_extension(target);
            if output_path.exists() {
                Ok(output_path)
            } else {
                Err(AppError::ConversionFailed(
                    "LibreOffice produced no output".into(),
                ))
            }
        }
    }
}

fn office_filter(target: &str) -> Option<&'static str> {
    match target {
        "pdf" => Some("pdf"),
        "docx" => Some("docx:Office Open XML Text"),
        "doc" => Some("doc:MS Word 97"),
        "rtf" => Some("rtf"),
        "odt" => Some("odt"),
        "xlsx" => Some("xlsx"),
        "xls" => Some("xls:MS Excel 97"),
        "ods" => Some("ods"),
        "pptx" => Some("pptx"),
        "ppt" => Some("ppt:MS PowerPoint 97"),
        "odp" => Some("odp"),
        _ => None,
    }
}
