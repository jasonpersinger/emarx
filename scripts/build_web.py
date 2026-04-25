#!/usr/bin/env python3
"""Prepare static EMARX web assets for hosting."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "web"
DATA = ROOT / "data"
WEB_DATA = WEB / "data"


def main() -> None:
    if WEB_DATA.exists():
        shutil.rmtree(WEB_DATA)
    (WEB_DATA / "works").mkdir(parents=True)
    (WEB_DATA / "metadata").mkdir(parents=True)

    catalog = []
    for meta_path in sorted((DATA / "metadata").glob("*.json")):
        metadata = json.loads(meta_path.read_text(encoding="utf-8"))
        work_path = DATA / "works" / meta_path.name
        work = json.loads(work_path.read_text(encoding="utf-8"))
        shutil.copy2(meta_path, WEB_DATA / "metadata" / meta_path.name)
        shutil.copy2(work_path, WEB_DATA / "works" / work_path.name)
        catalog.append(
            {
                "id": metadata["id"],
                "title": metadata["title"],
                "authors": metadata["authors"],
                "year": metadata["year"],
                "sections": len(work["sections"]),
                "paragraphs": sum(len(section["paragraphs"]) for section in work["sections"]),
                "source": metadata["source"],
                "source_url": metadata["source_url"],
                "license": metadata["license"],
                "translator": metadata.get("translator"),
                "aliases": metadata.get("aliases", []),
                "description": metadata.get("description", ""),
            }
        )

    catalog.sort(key=lambda item: (item["year"], item["title"]))
    (WEB_DATA / "catalog.json").write_text(
        json.dumps(catalog, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    print(f"Built web catalog with {len(catalog)} works.")


if __name__ == "__main__":
    main()
