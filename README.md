# SecurePDF backend

Rust/Axum service for converting supported documents and images to PDF.

## API contract

The versioned API contract is defined in [openapi.yaml](./openapi.yaml).

### Endpoints

- `GET /api/v1/health` reports service availability.
- `POST /api/v1/convert` accepts a `multipart/form-data` upload and returns an `application/pdf` binary response.

The conversion request must include:

- `file`: the source file
- `target_format`: `pdf`
- `options`: optional JSON string, such as `{"quality":"high","ocr":false}`

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
- `CORS_ORIGINS` (default `http://localhost:5173`)
- `CONVERSION_TIMEOUT_SECS` (default `60`)
