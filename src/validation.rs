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
    pub source_kind: FileKind,
}

#[derive(Debug, Clone)]
pub enum ConversionPath {
    Office { target: String },
    ImageToPdf,
    PdfToDocx,
    PdfToImage { target: String },
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
    if size == 0 {
        return Err(invalid("EMPTY_FILE", "The uploaded file is empty"));
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
                return Err(invalid("CORRUPT_FILE", "The file is not a valid PDF"));
            }
            let all_bytes = std::fs::read(path)?;
            if all_bytes
                .windows(b"/Encrypt".len())
                .any(|window| window == b"/Encrypt")
            {
                return Err(AppError::PasswordProtected);
            }
            FileKind::Pdf
        }
        "jpg" | "jpeg" | "webp" | "gif" | "bmp" => {
            let expected = match extension.as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                _ => unreachable!(),
            };
            if detected.as_deref() != Some(expected) {
                return Err(invalid(
                    "CORRUPT_FILE",
                    "The file contents do not match its extension",
                ));
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
        "ods" => validate_zip_office(&mut file, bytes, "content.xml", "ods")?,
        "pptx" => validate_zip_office(&mut file, bytes, "ppt/presentation.xml", "pptx")?,
        "odp" => validate_zip_office(&mut file, bytes, "content.xml", "odp")?,
        "odt" => validate_zip_office(&mut file, bytes, "mimetype", "odt")?,
        "rtf" => {
            if !bytes.starts_with(b"{\\rtf") {
                return Err(invalid(
                    "CORRUPT_FILE",
                    "The file is not a valid RTF document",
                ));
            }
            FileKind::Office
        }
        "doc" | "xls" | "ppt" => {
            if !bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
                return Err(invalid(
                    "CORRUPT_FILE",
                    "The file contents do not match its extension",
                ));
            }
            FileKind::Office
        }
        _ => {
            return Err(AppError::UnsupportedFormat {
                mime: format!(".{extension}"),
            });
        }
    };

    Ok(AcceptedFile {
        extension,
        source_kind: kind,
    })
}

pub fn validate_conversion(
    accepted: &AcceptedFile,
    target: &str,
) -> Result<ConversionPath, AppError> {
    let supported = match (accepted.source_kind, accepted.extension.as_str(), target) {
        (FileKind::Office, _, target)
            if matches!(
                target,
                "pdf"
                    | "docx"
                    | "doc"
                    | "rtf"
                    | "odt"
                    | "xlsx"
                    | "xls"
                    | "ods"
                    | "pptx"
                    | "ppt"
                    | "odp"
            ) =>
        {
            Some(ConversionPath::Office {
                target: target.into(),
            })
        }
        (FileKind::Image, "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp", "pdf") => {
            Some(ConversionPath::ImageToPdf)
        }
        (FileKind::Pdf, "pdf", "docx") => Some(ConversionPath::PdfToDocx),
        (FileKind::Pdf, "pdf", "png" | "jpg") => Some(ConversionPath::PdfToImage {
            target: target.into(),
        }),
        _ => None,
    };

    supported.ok_or_else(|| AppError::UnsupportedFormat {
        mime: format!(
            "conversion .{} -> .{} not supported",
            accepted.extension, target
        ),
    })
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
        .map_err(|_| invalid("CORRUPT_FILE", "The Office archive is malformed"))?;
    if archive.by_name(required_entry).is_err() {
        return Err(invalid(
            "CORRUPT_FILE",
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

    use super::{FileKind, validate_upload};

    #[test]
    fn accepts_pdf_magic_bytes() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(b"%PDF-1.7\n% valid test fixture with enough bytes\n")
            .expect("write PDF header");
        let accepted = validate_upload("document.pdf", file.path(), 49, 1024).expect("valid PDF");
        assert!(matches!(accepted.source_kind, FileKind::Pdf));
    }

    #[test]
    fn rejects_a_renamed_non_pdf() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(b"not a PDF").expect("write invalid header");
        let error = validate_upload("document.pdf", file.path(), 9, 1024)
            .expect_err("invalid PDF should be rejected");
        assert!(error.to_string().contains("valid PDF"));
    }

    #[test]
    fn rejects_empty_upload_with_a_specific_code() {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        let error = validate_upload("empty.pdf", file.path(), 0, 1024)
            .expect_err("empty file should be rejected");
        assert!(error.to_string().contains("empty"));
    }

    fn zip_with_entry(entry: &str) -> tempfile::NamedTempFile {
        let file = tempfile::NamedTempFile::new().expect("temp file");
        let mut archive = zip::ZipWriter::new(file.reopen().expect("reopen temp file"));
        archive
            .start_file(entry, zip::write::SimpleFileOptions::default())
            .expect("start zip entry");
        archive.write_all(b"<xml/>").expect("write zip entry");
        archive.finish().expect("finish zip");
        file
    }

    #[test]
    fn accepts_docx_with_document_entry() {
        let file = zip_with_entry("word/document.xml");
        let size = file.as_file().metadata().expect("metadata").len() as usize;
        let accepted =
            validate_upload("report.docx", file.path(), size, 1024 * 1024).expect("valid DOCX");
        assert!(matches!(accepted.source_kind, FileKind::Office));
    }

    #[test]
    fn rejects_docx_without_document_entry() {
        let file = zip_with_entry("other.xml");
        let size = file.as_file().metadata().expect("metadata").len() as usize;
        let error = validate_upload("report.docx", file.path(), size, 1024 * 1024)
            .expect_err("DOCX without content should be rejected");
        assert!(error.to_string().contains("docx"));
    }

    #[test]
    fn detects_password_protected_pdf() {
        let mut file = tempfile::NamedTempFile::new().expect("temp file");
        file.write_all(b"%PDF-1.7\n/Encrypt 7 0 R\ntrailer\n")
            .expect("write protected PDF marker");
        let error = validate_upload("protected.pdf", file.path(), 33, 1024)
            .expect_err("protected PDF should be rejected");
        assert!(matches!(error, crate::errors::AppError::PasswordProtected));
    }
}
