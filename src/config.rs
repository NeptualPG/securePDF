use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub max_upload_bytes: usize,
    pub temp_dir: String,
    pub libreoffice_path: String,
    pub imagemagick_path: String,
    pub pdftoppm_path: String,
    pub python_path: String,
    pub pdf2docx_script_path: String,
    pub cors_origins: Vec<String>,
    pub conversion_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(3000),
            max_upload_bytes: env::var("MAX_UPLOAD_MB")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(25)
                .saturating_mul(1024 * 1024),
            temp_dir: env::var("TEMP_DIR").unwrap_or_else(|_| "/tmp/pdf-converter".into()),
            libreoffice_path: env::var("LIBREOFFICE_PATH").unwrap_or_else(|_| "soffice".into()),
            imagemagick_path: env::var("IMAGEMAGICK_PATH").unwrap_or_else(|_| "convert".into()),
            pdftoppm_path: env::var("PDFTOPPM_PATH").unwrap_or_else(|_| "pdftoppm".into()),
            python_path: env::var("PYTHON_PATH").unwrap_or_else(|_| "python3".into()),
            pdf2docx_script_path: env::var("PDF2DOCX_SCRIPT_PATH")
                .unwrap_or_else(|_| "scripts/convert_pdf_to_docx.py".into()),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173".into())
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(String::from)
                .collect(),
            conversion_timeout_secs: env::var("CONVERSION_TIMEOUT_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(60),
        }
    }
}
