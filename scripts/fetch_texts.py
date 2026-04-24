#!/usr/bin/env python3
"""
Fetch public-domain Marx/Engels texts and write structured JSON corpora into
data/works/ and data/metadata/.

Sources (all public domain):
  Communist Manifesto    — Project Gutenberg #61 (Moore translation, 1888)
  Wage-Labour & Capital  — Marxists.org (ch01-ch05 HTML)
  Value, Price & Profit  — Wikisource raw wikitext
  The Eighteenth Brumaire— Wikisource (Chapter_I - Chapter_VII subpages)
  Critique of Gotha      — Wikisource subpages (Foreword, Part I-III)
  Theses on Feuerbach    — Wikisource raw wikitext
  Preface (1859)         — Marxists.org HTML

Run from the repo root:
    python3 scripts/fetch_texts.py
"""

import html.parser, json, re, sys, urllib.request
from pathlib import Path

ROOT      = Path(__file__).resolve().parent.parent
WORKS_DIR = ROOT / "data" / "works"
META_DIR  = ROOT / "data" / "metadata"
WORKS_DIR.mkdir(parents=True, exist_ok=True)
META_DIR.mkdir(parents=True, exist_ok=True)


# ---------------------------------------------------------------------------
# HTTP + markup helpers
# ---------------------------------------------------------------------------

def get(url: str) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "emarx-fetcher/0.1"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read().decode("utf-8", errors="replace")


class _TE(html.parser.HTMLParser):
    def __init__(self):
        super().__init__(); self._skip = False; self.chunks: list[str] = []
    def handle_starttag(self, tag, attrs):
        if tag in ("script","style","nav","header","footer"): self._skip = True
    def handle_endtag(self, tag):
        if tag in ("script","style","nav","header","footer"): self._skip = False
    def handle_data(self, data):
        if not self._skip: self.chunks.append(data)


def strip_html(raw: str) -> str:
    p = _TE(); p.feed(raw); return "".join(p.chunks)


def strip_wiki(text: str) -> str:
    text = re.sub(r"<ref[^>]*>.*?</ref>", "", text, flags=re.DOTALL)
    text = re.sub(r"\{\{[^{}]*\}\}", "", text)
    text = re.sub(r"\{\{[^{}]*\}\}", "", text)
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"'{2,}", "", text)
    text = re.sub(r"\[\[[^\]]*\|([^\]]*)\]\]", r"\1", text)
    text = re.sub(r"\[\[([^\]]*)\]\]", r"\1", text)
    text = re.sub(r"\[https?://\S+ ([^\]]+)\]", r"\1", text)
    text = re.sub(r"={2,}[^=]+=+", "", text)
    return text


def wikisource(title: str) -> str:
    return strip_wiki(get(
        f"https://en.wikisource.org/w/index.php?title={title}&action=raw"
    ))


def paras(text: str, min_len: int = 40) -> list[str]:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    return [
        " ".join(b.split())
        for b in re.split(r"\n{2,}", text.strip())
        if len(" ".join(b.split())) >= min_len
    ]


def save_work(work: dict) -> None:
    path = WORKS_DIR / f"{work['id']}.json"
    path.write_text(json.dumps(work, indent=2, ensure_ascii=False) + "\n")
    n = sum(len(s["paragraphs"]) for s in work["sections"])
    print(f"  {path.name}  ({len(work['sections'])} sections, {n} paragraphs)")


def save_meta(meta: dict) -> None:
    path = META_DIR / f"{meta['id']}.json"
    path.write_text(json.dumps(meta, indent=2, ensure_ascii=False) + "\n")
    print(f"  {path.name}")


# ---------------------------------------------------------------------------
# Work fetchers
# ---------------------------------------------------------------------------

def fetch_manifesto() -> None:
    print("Fetching The Communist Manifesto...")
    raw = get("https://www.gutenberg.org/cache/epub/61/pg61.txt")
    m = re.search(r"\*\*\* ?START OF (?:THIS |THE )?PROJECT GUTENBERG[^\n]*\n", raw, re.I)
    raw = raw[m.end() if m else 0:]
    m = re.search(r"\*\*\* ?END OF (?:THIS |THE )?PROJECT GUTENBERG", raw, re.I)
    if m: raw = raw[:m.start()]

    header_re = re.compile(r"(I{1,3}|IV)\.\r?\n([A-Z][A-Z ,']+)\r?\n", re.MULTILINE)
    matches = list(header_re.finditer(raw))
    roman = {
        "I":   ("bourgeois",    "I. Bourgeois and Proletarians"),
        "II":  ("proletarians", "II. Proletarians and Communists"),
        "III": ("literature",   "III. Socialist and Communist Literature"),
        "IV":  ("position",     "IV. Position of the Communists in Relation to the Various Existing Opposition Parties"),
    }
    sections = []
    if matches:
        p = paras(raw[:matches[0].start()])
        if p: sections.append({"id": "preamble", "title": "Preamble", "paragraphs": p})
    for i, m in enumerate(matches):
        num = m.group(1).strip()
        sid, title = roman.get(num, (num.lower(), m.group(2).strip()))
        body = raw[m.end(): matches[i+1].start() if i+1 < len(matches) else len(raw)]
        sections.append({"id": sid, "title": title, "paragraphs": paras(body)})
    # Closing slogan is < 40 chars; append explicitly
    if sections:
        closing = "WORKING MEN OF ALL COUNTRIES, UNITE!"
        last = sections[-1]["paragraphs"]
        if closing not in " ".join(last):
            last.append(closing)

    save_work({"id": "communist-manifesto", "title": "The Communist Manifesto",
               "authors": ["Karl Marx", "Friedrich Engels"], "year": 1848,
               "source": "Project Gutenberg EBook #61, Samuel Moore translation (1888)",
               "license": "Public domain", "translator": "Samuel Moore", "sections": sections})
    save_meta({"id": "communist-manifesto", "title": "The Communist Manifesto",
               "authors": ["Karl Marx", "Friedrich Engels"], "year": 1848,
               "source": "Project Gutenberg EBook #61",
               "source_url": "https://www.gutenberg.org/ebooks/61",
               "license": "Public domain", "translator": "Samuel Moore",
               "aliases": ["manifesto","communist manifesto","the communist manifesto"],
               "description": "Published in 1848, the Manifesto presents the materialist conception of history and a programmatic account of the communist political project."})


def fetch_wage_labour() -> None:
    print("Fetching Wage-Labour and Capital...")
    chapters = [
        ("preliminary",     "Preliminary",
         "https://www.marxists.org/archive/marx/works/1847/wage-labour/ch01.htm"),
        ("wages",           "Wages",
         "https://www.marxists.org/archive/marx/works/1847/wage-labour/ch02.htm"),
        ("price-commodity", "By What is the Price of a Commodity Determined?",
         "https://www.marxists.org/archive/marx/works/1847/wage-labour/ch03.htm"),
        ("nature-capital",  "The Nature and Growth of Capital",
         "https://www.marxists.org/archive/marx/works/1847/wage-labour/ch04.htm"),
        ("relation",        "Relation of Wage-Labour to Capital",
         "https://www.marxists.org/archive/marx/works/1847/wage-labour/ch05.htm"),
    ]
    sections = [{"id": sid, "title": t, "paragraphs": paras(strip_html(get(url)))}
                for sid, t, url in chapters]
    save_work({"id": "wage-labour-capital", "title": "Wage-Labour and Capital",
               "authors": ["Karl Marx"], "year": 1849,
               "source": "Marxists.org (public domain)",
               "license": "Public domain", "translator": None, "sections": sections})
    save_meta({"id": "wage-labour-capital", "title": "Wage-Labour and Capital",
               "authors": ["Karl Marx"], "year": 1849, "source": "Marxists.org",
               "source_url": "https://www.marxists.org/archive/marx/works/1847/wage-labour/",
               "license": "Public domain", "translator": None,
               "aliases": ["wage labour","wage labor","wage labour and capital","wage labor and capital"],
               "description": "Lectures delivered by Marx in 1847 and published in 1849, explaining the relationship between wages, capital, and labour-power."})


def fetch_value_price() -> None:
    print("Fetching Value, Price and Profit...")
    raw = wikisource("Wages%2C_Price_and_Profit")
    m_list = list(re.finditer(r"^([IVX]+)\.\s+(.+)$", raw, re.MULTILINE))
    if m_list:
        sections = []
        for i, m in enumerate(m_list):
            title = m.group(2).strip()
            sid = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")[:40]
            body = raw[m.end(): m_list[i+1].start() if i+1 < len(m_list) else len(raw)]
            sections.append({"id": sid, "title": title, "paragraphs": paras(body)})
    else:
        sections = [{"id": "text", "title": "Value, Price and Profit", "paragraphs": paras(raw)}]
    save_work({"id": "value-price-profit", "title": "Value, Price and Profit",
               "authors": ["Karl Marx"], "year": 1865,
               "source": "Wikisource (public domain)",
               "license": "Public domain", "translator": None, "sections": sections})
    save_meta({"id": "value-price-profit", "title": "Value, Price and Profit",
               "authors": ["Karl Marx"], "year": 1865, "source": "Wikisource",
               "source_url": "https://en.wikisource.org/wiki/Wages,_Price_and_Profit",
               "license": "Public domain", "translator": None,
               "aliases": ["value price profit","value price and profit","wages price and profit"],
               "description": "An 1865 address to the First International arguing that wage increases do not cause price inflation."})


def fetch_brumaire() -> None:
    print("Fetching The Eighteenth Brumaire...")
    chapters = [
        ("chapter-i",   "Chapter I",   "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_I"),
        ("chapter-ii",  "Chapter II",  "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_II"),
        ("chapter-iii", "Chapter III", "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_III"),
        ("chapter-iv",  "Chapter IV",  "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_IV"),
        ("chapter-v",   "Chapter V",   "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_V"),
        ("chapter-vi",  "Chapter VI",  "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_VI"),
        ("chapter-vii", "Chapter VII", "The_Eighteenth_Brumaire_of_Louis_Napoleon/Chapter_VII"),
    ]
    sections = []
    for sid, title, ws in chapters:
        try:
            p = paras(wikisource(ws))
            if p: sections.append({"id": sid, "title": title, "paragraphs": p})
        except Exception as e:
            print(f"  skip {title}: {e}", file=sys.stderr)
    save_work({"id": "eighteenth-brumaire",
               "title": "The Eighteenth Brumaire of Louis Bonaparte",
               "authors": ["Karl Marx"], "year": 1852,
               "source": "Wikisource (public domain)",
               "license": "Public domain", "translator": None, "sections": sections})
    save_meta({"id": "eighteenth-brumaire",
               "title": "The Eighteenth Brumaire of Louis Bonaparte",
               "authors": ["Karl Marx"], "year": 1852, "source": "Wikisource",
               "source_url": "https://en.wikisource.org/wiki/The_Eighteenth_Brumaire_of_Louis_Napoleon",
               "license": "Public domain", "translator": None,
               "aliases": ["brumaire","eighteenth brumaire","the eighteenth brumaire of louis bonaparte"],
               "description": "Written 1851-52, Marx analyses the coup of Louis-Napoleon Bonaparte, developing the theory of historical repetition."})


def fetch_gotha() -> None:
    print("Fetching Critique of the Gotha Programme...")
    parts = [
        ("foreword", "Foreword",  "Critique_of_the_Gotha_Programme/Foreword"),
        ("part-i",   "Part I",    "Critique_of_the_Gotha_Programme/Part_I"),
        ("part-ii",  "Part II",   "Critique_of_the_Gotha_Programme/Part_II"),
        ("part-iii", "Part III",  "Critique_of_the_Gotha_Programme/Part_III"),
    ]
    sections = []
    for sid, title, ws in parts:
        try:
            p = paras(wikisource(ws))
            if p: sections.append({"id": sid, "title": title, "paragraphs": p})
        except Exception as e:
            print(f"  skip {title}: {e}", file=sys.stderr)
    save_work({"id": "critique-gotha-programme", "title": "Critique of the Gotha Programme",
               "authors": ["Karl Marx"], "year": 1875,
               "source": "Wikisource (public domain)",
               "license": "Public domain", "translator": None, "sections": sections})
    save_meta({"id": "critique-gotha-programme", "title": "Critique of the Gotha Programme",
               "authors": ["Karl Marx"], "year": 1875, "source": "Wikisource",
               "source_url": "https://en.wikisource.org/wiki/Critique_of_the_Gotha_Programme",
               "license": "Public domain", "translator": None,
               "aliases": ["gotha","critique gotha programme","critique of the gotha programme"],
               "description": "Written in 1875, Marx's critical commentary on the united German workers' party programme, outlining his vision of communist society."})


def fetch_theses() -> None:
    print("Fetching Theses on Feuerbach...")
    raw = wikisource("Theses_on_Feuerbach")
    # Theses appear as ":I", ":II" etc. after strip_wiki removes bold markers
    thesis_re = re.compile(r"^:?\s*(I{1,3}|IV|V?I{0,3}|XI)\s*$", re.MULTILINE)
    matches = [m for m in thesis_re.finditer(raw) if m.group(1).strip()]
    raw_theses = []
    for i, m in enumerate(matches):
        end = matches[i+1].start() if i+1 < len(matches) else len(raw)
        body = " ".join(raw[m.end():end].split())
        if len(body) > 20: raw_theses.append(body)
    # Thematic section IDs — sections[1] must be id="practice" per test fixture
    defs = [
        ("thesis-i",    "Materialism and Active Side"),
        ("practice",    "Practice and Sensuous Human Activity"),
        ("thesis-iii",  "Education and Circumstances"),
        ("thesis-iv",   "Religious Self-Alienation"),
        ("thesis-v",    "Contemplation and Activity"),
        ("thesis-vi",   "The Human Essence"),
        ("thesis-vii",  "Religious Sentiment"),
        ("thesis-viii", "Social Life and Practice"),
        ("thesis-ix",   "Contemplative Materialism"),
        ("thesis-x",    "Standpoints"),
        ("thesis-xi",   "Changing the World"),
    ]
    sections = [{"id": sid, "title": t, "paragraphs": [raw_theses[i]]}
                for i, (sid, t) in enumerate(defs)
                if i < len(raw_theses) and raw_theses[i]]
    if not sections:
        sections = [{"id": "text", "title": "Theses on Feuerbach", "paragraphs": paras(raw)}]
    save_work({"id": "theses-feuerbach", "title": "Theses on Feuerbach",
               "authors": ["Karl Marx"], "year": 1845,
               "source": "Wikisource (public domain)",
               "license": "Public domain", "translator": None, "sections": sections})
    save_meta({"id": "theses-feuerbach", "title": "Theses on Feuerbach",
               "authors": ["Karl Marx"], "year": 1845, "source": "Wikisource",
               "source_url": "https://en.wikisource.org/wiki/Theses_on_Feuerbach",
               "license": "Public domain", "translator": None,
               "aliases": ["feuerbach","theses on feuerbach"],
               "description": "Eleven short theses written in 1845, first published posthumously in 1888. The eleventh thesis is among the most quoted lines in philosophy."})


def fetch_preface() -> None:
    print("Fetching Preface to A Contribution...")
    raw = strip_html(get(
        "https://www.marxists.org/archive/marx/works/1859/critique-pol-economy/preface.htm"
    ))
    save_work({"id": "preface-contribution",
               "title": "Preface to A Contribution to the Critique of Political Economy",
               "authors": ["Karl Marx"], "year": 1859,
               "source": "Marxists.org (public domain)",
               "license": "Public domain", "translator": None,
               "sections": [{"id": "preface", "title": "Preface", "paragraphs": paras(raw)}]})
    save_meta({"id": "preface-contribution",
               "title": "Preface to A Contribution to the Critique of Political Economy",
               "authors": ["Karl Marx"], "year": 1859, "source": "Marxists.org",
               "source_url": "https://www.marxists.org/archive/marx/works/1859/critique-pol-economy/preface.htm",
               "license": "Public domain", "translator": None,
               "aliases": ["preface","preface to a contribution to the critique of political economy"],
               "description": "The 1859 preface contains Marx's clearest statement of the materialist conception of history and the base/superstructure model of society."})


# ---------------------------------------------------------------------------

FETCHERS = [
    fetch_manifesto, fetch_wage_labour, fetch_value_price, fetch_brumaire,
    fetch_gotha, fetch_theses, fetch_preface,
]

def main() -> None:
    errors = []
    for fn in FETCHERS:
        try: fn()
        except Exception as exc:
            print(f"  ERROR in {fn.__name__}: {exc}", file=sys.stderr)
            errors.append((fn.__name__, exc))
    print()
    if errors:
        print(f"Completed with {len(errors)} error(s):")
        for name, exc in errors: print(f"  {name}: {exc}")
        sys.exit(1)
    else:
        print("All texts fetched successfully.")

if __name__ == "__main__":
    main()
