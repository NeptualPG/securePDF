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

The full stack (backend + frontend) is defined in `docker-compose.yml` of the `securePDF-Frontend` repository, cloned next to this one:

```bash
cd ../securePDF-Frontend
docker compose up --build   # http://localhost:8080
```

To run only the backend image:

```bash
docker build -t securepdf-api .
docker run --rm -p 3000:3000 securepdf-api
```

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

## Calidad de código

Requisitos:

- [rustup](https://rustup.rs). La versión (1.90, con `rustfmt` y `clippy`) está fijada en `rust-toolchain.toml` y se instala sola al ejecutar `cargo`.
- [pre-commit](https://pre-commit.com), por ejemplo `uv tool install pre-commit`.
- `cargo install --locked cargo-audit cargo-machete`.

| Comando                                                    | Qué hace                                                     |
| ---------------------------------------------------------- | ------------------------------------------------------------ |
| `cargo fmt --check`                                        | Comprueba el formato                                         |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lint; cualquier warning falla                                |
| `cargo test`                                               | Tests de validación y del router                             |
| `cargo build --release`                                    | Build de producción                                          |
| `cargo audit -D warnings`                                  | Vulnerabilidades y crates retirados (yanked) en `Cargo.lock` |
| `cargo machete`                                            | Dependencias sin usar                                        |

`Cargo.toml` prohíbe `unsafe` (`unsafe_code = "forbid"`).

### Hooks de pre-commit

```bash
pre-commit install            # una vez por clon
pre-commit run --all-files    # ejecutar todos los hooks a mano
```

En cada commit se ejecutan:

- Hooks genéricos: espacios al final de línea, salto de línea final, marcadores de conflicto, YAML/TOML válidos, archivos de más de 1 MB y claves privadas.
- `cargo fmt` (formatea los `.rs` en stage), `clippy`, `cargo test`, `cargo build --release` y `cargo machete` cuando cambian `src/` o `Cargo.*`.
- `cargo audit` cuando cambian `Cargo.toml` o `Cargo.lock` (necesita red).

Si un hook modifica archivos (por ejemplo `cargo fmt`), el commit se detiene: revisa los cambios, haz `git add` y vuelve a hacer commit.

Problemas comunes:

- `cargo: command not found` dentro del hook: añade `~/.cargo/bin` al `PATH` (`source ~/.cargo/env`).
- `cargo audit` falla sin conexión: reintenta con red. No uses `--no-verify` salvo en una emergencia.
- `error: no such command: audit` o `machete`: instala las herramientas de los requisitos.

## Despliegue en Render

`render.yaml` define el servicio Docker `securepdf-api` (plan free, región oregon, health check en `/api/v1/health`, `MAX_UPLOAD_MB=25`). Render inyecta `PORT`.

1. En Render: **New → Blueprint** y elige este repositorio.
2. La URL pública será `https://securepdf-api.onrender.com`. Si Render asigna otra, actualízala en `netlify.toml` del frontend.

El plan free se duerme tras 15 minutos sin tráfico y tiene 512 MB de RAM; LibreOffice puede quedarse corto con documentos grandes.
