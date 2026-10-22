# SecurePDF backend

Rust/Axum service for converting supported documents and images to and from PDF.

## API contract

The versioned API contract is defined in [openapi.yaml](./openapi.yaml).

### Endpoints

- `GET /api/v1/health` reports service availability.
- `POST /api/v1/convert` accepts a `multipart/form-data` upload and returns PDF, DOCX, PNG, or JPEG binary output.

The conversion request must include:

- `file`: the source file
- `target_format`: `pdf`, `docx`, `png`, or `jpg`
- `options`: optional JSON string. PDF-to-image supports `page` and `dpi`; PDF-to-DOCX supports `preserve_layout`.

Supported conversions:

- Office documents and images to PDF
- PDF to DOCX (best effort)
- PDF to PNG or JPG, one page at a time

Errors use this JSON shape:

```json
{
  "error": "A valid file is required",
  "code": "INVALID_FILE"
}
```

The implementation is expected to return `400` for invalid uploads, `413` for requests over the upload limit, and `422` for unsupported source formats.

## Run with Docker

From the parent directory:

```powershell
docker compose up --build
```

The backend is available at `http://localhost:3000`.

### Configuration

The service reads these environment variables:

- `PORT` (default `3000`)
- `MAX_UPLOAD_MB` (default `25`)
- `TEMP_DIR` (default `/tmp/pdf-converter`)
- `LIBREOFFICE_PATH` (default `soffice`)
- `IMAGEMAGICK_PATH` (default `convert`)
- `PDFTOPPM_PATH` (default `pdftoppm`)
- `PYTHON_PATH` (default `python3`)
- `PDF2DOCX_SCRIPT_PATH` (default `scripts/convert_pdf_to_docx.py`)
- `CORS_ORIGINS` (default `http://localhost:5173`)
- `CONVERSION_TIMEOUT_SECS` (default `60`)
