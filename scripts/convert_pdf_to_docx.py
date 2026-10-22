import sys

from pdf2docx import Converter


def main() -> None:
    if len(sys.argv) != 3:
        raise SystemExit("usage: convert_pdf_to_docx.py INPUT OUTPUT")

    converter = Converter(sys.argv[1])
    try:
        converter.convert(sys.argv[2])
    finally:
        converter.close()


if __name__ == "__main__":
    main()
