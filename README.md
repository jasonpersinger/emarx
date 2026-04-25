# EMARX

**Read Marx from the command line.**

EMARX is an offline-first Rust CLI/TUI reader for public-domain and freely
redistributable works by Karl Marx, Friedrich Engels, and related texts. It is
made for people who want a serious terminal reading tool: fast browsing, clean
plain text output, source metadata, search, bookmarks, themes, and a compact
full-screen reader.

Website: <https://emarx.space>

Repository: <https://github.com/jasonpersinger/emarx>

## What It Is

EMARX is a single executable named `emarx`.

Run it with no arguments and it opens a full-screen terminal reader with three
panels: Works, Sections, and Text. Use it in scripts or pipes and it prints
copy-friendly plain text.

It is offline-first. Normal reading, search, source lookup, bookmarks, and
resume behavior do not require network access, API keys, accounts, or a web
service.

## Install

Install from GitHub with Cargo:

```sh
cargo install --git https://github.com/jasonpersinger/emarx --force
```

Then run:

```sh
emarx
```

Requirements:

- Rust and Cargo
- A terminal with basic Unicode support
- Linux, macOS, or Windows

If Cargo installs successfully but your shell cannot find `emarx`, make sure
Cargo's bin directory is on your `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

For Fish:

```fish
fish_add_path -U ~/.cargo/bin
```

## Quick Commands

```text
emarx                         Launch the TUI reader
emarx list                    List bundled works
emarx read manifesto          Read a full work
emarx read manifesto 1        Read section 1
emarx read "capital vol 1" chapter 1
emarx search "commodity"      Search all bundled texts
emarx random                  Print a random passage
emarx today                   Print a deterministic passage of the day
emarx info manifesto          Show source/license metadata
emarx sources                 List bundled source URLs and license notes
emarx config                  Show config path and current settings
emarx intro                   Replay the startup screen
```

Reference aliases are forgiving. These all resolve as expected:

```text
manifesto
communist manifesto
capital
das kapital
capital 1
capital vol 1
gotha
brumaire
wage labour
value price profit
feuerbach
```

## TUI Controls

```text
Up/Down        Navigate the active panel
Left/Right     Switch panels
Tab/Shift-Tab  Switch panels
Enter          Select work/section or focus text
PageUp/Down    Scroll text by a page
Home/End       Jump to start/end of active panel
/              Search within current work
n / N          Next/previous search result
t              Cycle theme
b              Toggle bookmark
B              Open bookmark browser
g              Jump to section number
h / ?          Show help
q              Quit from the main reader
Esc / q        Close overlays such as help/bookmarks
```

## Features

- Full-screen TUI built with Ratatui and Crossterm
- Three-pane reader: Works, Sections, Text
- Offline bundled corpus
- Source and license metadata for each bundled work
- Full-text search over titles, section titles, and paragraphs
- Persistent reading position
- Bookmarks
- Passage of the day
- Random passage
- Theme support:
  - Dialectic Dark
  - Reading Room
  - Red Banner
  - Parchment
  - Monochrome
- Copy-friendly plain text command output
- Static website with a contained browser reader demo

## Bundled Starter Corpus

The current starter corpus is intentionally small and auditable:

- The Communist Manifesto
- Wage-Labour and Capital
- Value, Price and Profit
- Critique of the Gotha Programme
- The Eighteenth Brumaire of Louis Bonaparte
- Theses on Feuerbach
- Preface to A Contribution to the Critique of Political Economy
- Capital, Volume I

Structured texts live in `data/works/`. Metadata lives in `data/metadata/`.

## Copyright And Sources

EMARX is intended to bundle only public-domain or clearly redistributable texts.
Every bundled work should include:

- Source URL
- Author
- Translator/editor, if known
- Original publication year, if known
- License or public-domain note

Preferred sources include:

- Project Gutenberg public-domain editions
- Wikisource pages with compatible licensing
- Marxists Internet Archive pages that clearly identify public-domain or freely
  redistributable text

Avoid bundling:

- Modern copyrighted translations
- Marx/Engels Collected Works edition text
- Modern introductions, annotations, publisher notes, or editorial apparatus
  unless redistribution is explicitly permitted

See [DISCLAIMER.md](DISCLAIMER.md). Copyright status varies by jurisdiction.

## Development

Clone the repository:

```sh
git clone https://github.com/jasonpersinger/emarx.git
cd emarx
```

Run locally:

```sh
cargo run
```

Run CLI commands during development:

```sh
cargo run -- list
cargo run -- read manifesto
cargo run -- search "commodity fetishism"
```

Run tests:

```sh
cargo test
```

Build a release binary:

```sh
cargo build --release
```

The executable will be at:

```text
target/release/emarx
```

## Website

The static website is in `web/`. It uses generated JSON derived from the bundled
corpus.

Build the web catalog:

```sh
python3 scripts/build_web.py
```

Serve locally:

```sh
python3 -m http.server 4173 -d web
```

Then open:

```text
http://127.0.0.1:4173
```

## Importing More Texts

Use `scripts/import_text.py` to convert plain text into EMARX's structured JSON
format:

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

Then add corresponding metadata in `data/metadata/new-work.json` and register
the files in `src/library.rs`.

Keep the corpus boring and verifiable. Good provenance matters more than
quantity.

## License

The EMARX source code is licensed under the MIT License. Bundled texts retain
their own source/license status as documented in the metadata files.

WORKING PEOPLE OF ALL COUNTRIES, UNITE!
