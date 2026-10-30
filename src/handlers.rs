use axum::{
    extract::{Multipart, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{config::Config, conversion, errors::AppError, validation};

#[derive(Debug, Default, Deserialize)]
struct ConversionOptions {
    page: Option<u32>,
    dpi: Option<u32>,
}

pub async fn convert(
    State(config): State<Config>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let mut target_format = None;
    let mut options = ConversionOptions::default();
    let temp_dir = tempfile::TempDir::new_in(&config.temp_dir)?;
    let mut input_path = None;
    let mut filename = None;
    let mut file_seen = false;
    let mut size = 0_usize;

    while let Some(mut field) = multipart.next_field().await? {
        match field.name().unwrap_or_default() {
            "file" => {
                if file_seen {
                    return Err(AppError::InvalidFile {
                        code: "DUPLICATE_FILE".into(),
                        detail: "only one 'file' field is allowed".into(),
                    });
                }
                file_seen = true;
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
<<<<<<< HEAD
                size = 0;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|error| AppError::Multipart(error.to_string()))?
                {
=======
                while let Some(chunk) = field.chunk().await? {
>>>>>>> c4f309b97d75aec7c946a9085e09632e0b596e33
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
                target_format = Some(field.text().await?);
            }
            "options" => {
                let options_json = field.text().await?;
                if !options_json.trim().is_empty() {
                    options =
                        serde_json::from_str(&options_json).map_err(|_| AppError::InvalidFile {
                            code: "INVALID_OPTIONS".into(),
                            detail: "options must be valid JSON".into(),
                        })?;
                }
            }
            _ => while field.chunk().await?.is_some() {},
        }
    }

    let filename = filename.ok_or_else(|| AppError::InvalidFile {
        code: "MISSING_FILE".into(),
        detail: "A 'file' field is required".into(),
    })?;
    let input_path = input_path.ok_or_else(|| AppError::InvalidFile {
        code: "MISSING_FILE".into(),
        detail: "A 'file' field is required".into(),
    })?;
    // Validation reads the file with blocking I/O, so keep it off the async workers.
    let accepted = {
        let (filename, input_path) = (filename.clone(), input_path.clone());
        let max_upload_bytes = config.max_upload_bytes;
        tokio::task::spawn_blocking(move || {
            validation::validate_upload(&filename, &input_path, size, max_upload_bytes)
        })
        .await
        .map_err(|error| AppError::ConversionFailed(error.to_string()))??
    };
    let target = target_format
        .map(|value| value.trim().to_ascii_lowercase())
        .ok_or_else(|| AppError::InvalidFile {
            code: "MISSING_FIELD".into(),
            detail: "target_format is required".into(),
        })?;
    let path = validation::validate_conversion(&accepted, &target)?;
    let output_path = match path {
        validation::ConversionPath::Office { target } => {
            tracing::info!(extension = %accepted.extension, "converting office document");
            conversion::office::convert(&input_path, &target, &config).await?
        }
        validation::ConversionPath::ImageToPdf => {
            conversion::images::convert(&input_path, &config).await?
        }
        validation::ConversionPath::PdfToDocx => {
            conversion::pdf_to_docx::convert(&input_path, &config).await?
        }
        validation::ConversionPath::PdfToImage { target } => {
            let page = options.page.unwrap_or(1).max(1);
            let dpi = options.dpi.unwrap_or(150).clamp(72, 600);
            conversion::pdf_to_image::convert(&input_path, &target, page, dpi, &config).await?
        }
    };
    let (content_type, filename) = match target.as_str() {
        "pdf" => ("application/pdf", "converted.pdf"),
        "docx" => (
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "converted.docx",
        ),
        "png" => ("image/png", "converted.png"),
        "jpg" => ("image/jpeg", "converted.jpg"),
        "doc" => ("application/msword", "converted.doc"),
        "odt" => ("application/vnd.oasis.opendocument.text", "converted.odt"),
        "xls" => ("application/vnd.ms-excel", "converted.xls"),
        "xlsx" => (
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "converted.xlsx",
        ),
        "ods" => (
            "application/vnd.oasis.opendocument.spreadsheet",
            "converted.ods",
        ),
        "ppt" => ("application/vnd.ms-powerpoint", "converted.ppt"),
        "pptx" => (
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "converted.pptx",
        ),
        "odp" => (
            "application/vnd.oasis.opendocument.presentation",
            "converted.odp",
        ),
        "rtf" => ("application/rtf", "converted.rtf"),
        _ => {
            return Err(AppError::UnsupportedFormat {
                mime: format!("target_format={target}"),
            });
        }
    };
    let bytes = tokio::fs::read(output_path).await?;

    let mut response = bytes.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
            .map_err(|error| AppError::ConversionFailed(error.to_string()))?,
    );
    Ok(response)
}

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
