FROM rust:1.90-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && printf 'fn main() {}\n' > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        ca-certificates \
        wget \
        libreoffice-core \
        libreoffice-writer \
        libreoffice-calc \
        libreoffice-impress \
        imagemagick \
        poppler-utils \
        python3 \
        python3-pip \
    && rm -rf /var/lib/apt/lists/*

RUN pip3 install --no-cache-dir --break-system-packages pdf2docx

COPY --from=builder /app/target/release/securePDF /usr/local/bin/securePDF
COPY scripts/convert_pdf_to_docx.py /app/scripts/convert_pdf_to_docx.py

ENV PDF2DOCX_SCRIPT_PATH=/app/scripts/convert_pdf_to_docx.py

EXPOSE 3000
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/securePDF"]
