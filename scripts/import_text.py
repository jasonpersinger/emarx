#!/usr/bin/env python3
"""Convert a plain text source into EMARX work JSON.

Sections begin whenever a line starts with --section-marker. Paragraphs are
formed from blank-line separated text within each section.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def slugify(value: str) -> str:
    value = re.sub(r"[^a-zA-Z0-9]+", "-", value.lower()).strip("-")
    return value or "section"


def paragraphs(lines: list[str]) -> list[str]:
    blocks: list[str] = []
    current: list[str] = []
    for line in lines:
        stripped = line.strip()
        if not stripped:
            if current:
                blocks.append(" ".join(current))
                current = []
            continue
        current.append(stripped)
    if current:
        blocks.append(" ".join(current))
    return blocks


def parse_sections(raw: str, marker: str) -> list[dict[str, object]]:
    sections: list[dict[str, object]] = []
    title = "Text"
    buffer: list[str] = []

    for line in raw.splitlines():
        if line.startswith(marker):
            if buffer:
                sections.append(
                    {
                        "id": slugify(title),
                        "title": title,
                        "paragraphs": paragraphs(buffer),
                    }
                )
            title = line[len(marker) :].strip() or "Untitled"
            buffer = []
        else:
            buffer.append(line)

    if buffer or not sections:
        sections.append(
            {"id": slugify(title), "title": title, "paragraphs": paragraphs(buffer)}
        )

    return sections


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("--id", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--author", action="append", required=True)
    parser.add_argument("--year", type=int, required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--license", required=True)
    parser.add_argument("--translator")
    parser.add_argument("--section-marker", default="## ")
    args = parser.parse_args()

    raw = args.input.read_text(encoding="utf-8")
    work = {
        "id": args.id,
        "title": args.title,
        "authors": args.author,
        "year": args.year,
        "source": args.source,
        "license": args.license,
        "translator": args.translator,
        "sections": parse_sections(raw, args.section_marker),
    }
    print(json.dumps(work, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
