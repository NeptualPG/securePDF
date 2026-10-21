use axum::{
    extract::{Multipart, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{config::Config, conversion, errors::AppError, validation};

pub async fn convert(
    State(config): State<Config>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let mut target_format = None;
    let temp_dir = tempfile::TempDir::new_in(&config.temp_dir)?;
    let mut input_path = None;
    let mut filename = None;
    let mut size = 0_usize;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::Multipart(error.to_string()))?
    {
        match field.name().unwrap_or_default() {
            "file" => {
                let original_name = field
                    .file_name()
                    .ok_or_else(|| AppError::InvalidFile {
                        code: "MISSING_FILENAME".into(),
                        detail: "The uploaded file must have a name".into(),
                    })?
                    .to_string();
                let extension = std::path::Path::new(&original_name)
                    .extension()
                    .and_then(|value| value.to_str())
                    .unwrap_or("bin")
                    .to_ascii_lowercase();
                let path = temp_dir
                    .path()
                    .join(format!("input-{}.{}", Uuid::new_v4(), extension));
                let mut input = tokio::fs::File::create(&path).await?;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|error| AppError::Multipart(error.to_string()))?
                {
                    size = size.saturating_add(chunk.len());
                    if size > config.max_upload_bytes {
                        return Err(AppError::PayloadTooLarge);
                    }
                    input.write_all(&chunk).await?;
                }
                input.flush().await?;
                filename = Some(original_name);
                input_path = Some(path);
            }
            "target_format" => {
                target_format = Some(
                    field
                        .text()
                        .await
                        .map_err(|error| AppError::Multipart(error.to_string()))?,
                );
            }
            "options" => {
                let options = field
                    .text()
                    .await
                    .map_err(|error| AppError::Multipart(error.to_string()))?;
                if !options.trim().is_empty() {
                    serde_json::from_str::<serde_json::Value>(&options).map_err(|_| {
                        AppError::InvalidFile {
                            code: "INVALID_OPTIONS".into(),
                            detail: "options must be valid JSON".into(),
                        }
                    })?;
                }
            }
            _ => {
                while field
                    .chunk()
                    .await
                    .map_err(|error| AppError::Multipart(error.to_string()))?
                    .is_some()
                {}
            }
        }
    }

    if target_format.as_deref() != Some("pdf") {
        return Err(AppError::UnsupportedFormat {
            mime: format!(
                "target_format={}",
                target_format.unwrap_or_else(|| "missing".into())
            ),
        });
    }

    let filename = filename.ok_or_else(|| AppError::InvalidFile {
        code: "MISSING_FILE".into(),
        detail: "A 'file' field is required".into(),
    })?;
    let input_path = input_path.ok_or_else(|| AppError::InvalidFile {
        code: "MISSING_FILE".into(),
        detail: "A 'file' field is required".into(),
    })?;
    let accepted =
        validation::validate_upload(&filename, &input_path, size, config.max_upload_bytes)?;
    let output_path = match accepted.kind {
        validation::FileKind::Pdf => conversion::pdf::convert(&input_path).await?,
        validation::FileKind::Office => {
            tracing::info!(extension = %accepted.extension, "converting office document");
            conversion::office::convert(&input_path, &config).await?
        }
        validation::FileKind::Image => conversion::images::convert(&input_path, &config).await?,
    };
    let bytes = tokio::fs::read(output_path).await?;

    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"converted.pdf\""),
    );
    Ok(response)
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
