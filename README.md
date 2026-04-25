# EMARX

Read Marx from the command line.

EMARX is an offline-first terminal reader for public-domain and freely redistributable works by Karl Marx and Friedrich Engels. It provides a full-screen TUI for browsing, reading, searching, bookmarking, and resuming texts, plus copy-friendly plain text CLI output for scripts and pipes.

Web reader: <https://emarx.netlify.app>  
Repository: <https://github.com/jasonpersinger/emarx>

## Features

- Single Rust binary named `emarx`
- Full-screen Ratatui/Crossterm TUI when run with no arguments
- Three-panel browser: works, sections, and text
- Fast in-memory full-text search over titles, section titles, and paragraphs
- Persistent reading position and bookmarks
- Theme support: Dialectic Dark, Reading Room, Red Banner, Parchment, Monochrome
- Offline bundled starter corpus with per-work metadata
- Plain text output for `read`, `search`, `random`, `today`, `info`, and `sources`
- Static retro terminal-style web reader deployable to Netlify

## Install and Run

Install from GitHub:

```sh
cargo install --git https://github.com/jasonpersinger/emarx
emarx
```

Install from a local checkout:

```sh
cargo install --path .
emarx
```

For development:

```sh
cargo run
cargo run -- list
cargo run -- read manifesto
cargo run -- read manifesto 1
cargo run -- read "capital vol 1" chapter 1
cargo run -- search "commodity"
cargo test
```

Build the static web reader:

```sh
python3 scripts/build_web.py
python3 -m http.server 4173 -d web
```

## Commands

```text
emarx                         Launch the TUI
emarx list                    List bundled works
emarx read manifesto          Read a full work
emarx read manifesto 1        Read section 1
emarx read "capital vol 1" chapter 1
emarx search "commodity"      Search all bundled texts
emarx random                  Print a random passage
emarx today                   Print a deterministic passage of the day
emarx info manifesto          Show source/license metadata
emarx sections capital        List chapters/sections for a work
emarx stats                   Show corpus counts
emarx bookmarks               List saved bookmarks
emarx bookmarks add capital 15
emarx bookmarks remove 1
emarx bookmarks clear
emarx sources                 List bundled source URLs and license notes
emarx intro                   Replay startup banner
emarx config                  Show config path and current settings
```

Forgiving aliases include `manifesto`, `communist manifesto`, `capital`, `das kapital`, `capital 1`, `capital vol 1`, `gotha`, `brumaire`, `wage labour`, `value price profit`, and `feuerbach`.

## TUI Keys

```text
Up/Down        Navigate current panel
Left/Right     Switch panel
Tab/Shift-Tab  Switch panel
Enter          Select
PageUp/Down    Scroll text by a page
Home/End       Jump to start/end of active panel
/              Search within current work
n / N          Next/previous search result
t              Cycle theme
b              Toggle bookmark
B              Open bookmark browser
g              Jump to section number
?              Show help overlay
q              Quit
```

## Bundled Starter Corpus

The current starter corpus is structured as JSON under `data/works/` with matching metadata under `data/metadata/`.

- The Communist Manifesto
- Wage Labour and Capital
- Value, Price and Profit
- Critique of the Gotha Programme
- The Eighteenth Brumaire of Louis Bonaparte
- Theses on Feuerbach
- Preface to A Contribution to the Critique of Political Economy
- Capital, Volume I

This repository intentionally keeps the corpus small and auditable at first. Expand it by importing only public-domain or clearly redistributable text, then add matching metadata with source URL, translator/editor if known, publication date if known, and a license/public-domain note.

## Adding Texts Safely

Preferred sources:

- Project Gutenberg public-domain editions
- Wikisource pages with compatible licensing
- Marxists Internet Archive pages that clearly identify public-domain or freely redistributable texts

Avoid bundling:

- Modern copyrighted translations
- Marx/Engels Collected Works edition text
- Modern introductions, annotations, footnotes, publisher notes, or editorial apparatus unless redistribution is explicitly permitted

Use `scripts/import_text.py` to convert a plain text file into the structured JSON shape used by EMARX:

```sh
python3 scripts/import_text.py source.txt \
  --id new-work \
  --title "New Work" \
  --author "Karl Marx" \
  --year 1860 \
  --source "Project Gutenberg eBook #..." \
  --license "Public domain in the United States" \
  --section-marker "## " \
  > data/works/new-work.json
```

Then create `data/metadata/new-work.json` and add both files to the embedded arrays in `src/library.rs`.

## Release Notes

Build release binaries with:

```sh
cargo build --release
```

The executable will be at:

```text
target/release/emarx
```

For distribution, build on each target platform or use a cross-platform release tool such as `cargo-dist`. Normal use requires no network access and no API keys.

## Copyright

See [DISCLAIMER.md](DISCLAIMER.md). Copyright status varies by jurisdiction. EMARX is intended to bundle only public-domain or freely redistributable texts with documented provenance.
