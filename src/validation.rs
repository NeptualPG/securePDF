use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::errors::AppError;

#[derive(Debug, Clone, Copy)]
pub enum FileKind {
    Office,
    Image,
    Pdf,
}

#[derive(Debug)]
pub struct AcceptedFile {
    pub extension: String,
    pub kind: FileKind,
}

pub fn validate_upload(
    file_name: &str,
    path: &Path,
    size: usize,
    max_size: usize,
) -> Result<AcceptedFile, AppError> {
    if size > max_size {
        return Err(AppError::PayloadTooLarge);
    }

    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| invalid("INVALID_FILE", "A file extension is required"))?;

    let mut file = File::open(path)?;
    let mut header = [0_u8; 8192];
    let header_size = file.read(&mut header)?;
    let bytes = &header[..header_size];
    let detected = infer::get(bytes).map(|kind| kind.mime_type().to_string());
    let kind = match extension.as_str() {
        "pdf" => {
            if !bytes.starts_with(b"%PDF-") {
                return Err(invalid("INVALID_FILE", "The file is not a valid PDF"));
            }
            FileKind::Pdf
        }
        "jpg" | "jpeg" => {
            if detected.as_deref() != Some("image/jpeg") {
                return Err(invalid("INVALID_FILE", "The file is not a valid JPEG image"));
            }
            FileKind::Image
        }
        "png" => {
            if detected.as_deref() != Some("image/png") {
                return Err(invalid("INVALID_FILE", "The file is not a valid PNG image"));
            }
            FileKind::Image
        }
        "docx" => validate_zip_office(&mut file, bytes, "word/document.xml", "docx")?,
        "xlsx" => validate_zip_office(&mut file, bytes, "xl/workbook.xml", "xlsx")?,
        "pptx" => validate_zip_office(&mut file, bytes, "ppt/presentation.xml", "pptx")?,
        "odt" => validate_zip_office(&mut file, bytes, "mimetype", "odt")?,
        "doc" | "xls" | "ppt" => {
            if !bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
                return Err(invalid("INVALID_FILE", "The file is not a valid Office document"));
            }
            FileKind::Office
        }
        _ => {
            return Err(AppError::UnsupportedFormat {
                mime: format!(".{extension}"),
            });
        }
    };

    Ok(AcceptedFile { extension, kind })
}

fn validate_zip_office(
    file: &mut File,
    bytes: &[u8],
    required_entry: &str,
    extension: &str,
) -> Result<FileKind, AppError> {
    if !bytes.starts_with(b"PK") {
        return Err(invalid(
            "INVALID_FILE",
            "The file is not a valid ZIP-based Office document",
        ));
    }
    file.seek(SeekFrom::Start(0))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|_| invalid("INVALID_FILE", "The Office archive is malformed"))?;
    if archive.by_name(required_entry).is_err() {
        return Err(invalid(
            "INVALID_FILE",
            &format!("The file does not contain the required {extension} content"),
        ));
    }
    Ok(FileKind::Office)
}

fn invalid(code: &str, detail: &str) -> AppError {
    AppError::InvalidFile {
        code: code.into(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::{validate_upload, FileKind};

    #[test]
    fn accepts_pdf_magic_bytes() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(b"%PDF-1.7\n").expect("write PDF header");
        let accepted =
            validate_upload("document.pdf", file.path(), 9, 1024).expect("valid PDF");
        assert!(matches!(accepted.kind, FileKind::Pdf));
    }

    #[test]
    fn rejects_a_renamed_non_pdf() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(b"not a PDF").expect("write invalid header");
        let error = validate_upload("document.pdf", file.path(), 9, 1024)
            .expect_err("invalid PDF should be rejected");
        assert!(error.to_string().contains("valid PDF"));
    }
}
