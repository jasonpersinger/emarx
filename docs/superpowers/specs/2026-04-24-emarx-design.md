# emarx — Design Spec

**Date:** 2026-04-24  
**Status:** Approved

## Overview

`emarx` is a terminal-based reader for the core Marx/Engels canon, built in the spirit of [christ-cli](https://github.com/whoisyurii/christ-cli). A single Rust binary that lets developers read Marx without leaving the command line.

## Tech Stack

- **Language:** Rust (2021 edition)
- **TUI:** ratatui + crossterm
- **Animations:** tachyonfx
- **CLI parsing:** clap (derive)
- **Async runtime:** tokio
- **Config/persistence:** confy + directories
- **Serialization:** serde + serde_json
- **Distribution:** single binary via curl install script + crates.io

## Bundled Texts (offline, static data)

All three texts are public domain and bundled at compile time:

1. **The Communist Manifesto** (Marx & Engels, 1848) — 4 sections + Preamble
2. **Theses on Feuerbach** (Marx, 1845) — 11 numbered theses
3. **Capital, Volume 1** (Marx, 1867) — 8 parts, 33 chapters

Texts are stored as structured JSON in `data/` and embedded via `include_str!` at compile time. No internet required for core functionality.

## TUI — 3-Panel Browser

```
┌─────────────┬──────────────────┬────────────────────────────────┐
│    Works    │    Chapters      │            Text                │
│─────────────│──────────────────│────────────────────────────────│
│ Manifesto   │ Preamble         │ A spectre is haunting Europe   │
│ Theses      │ I. Bourgeois...  │ — the spectre of communism.    │
│ Capital I   │ II. Proletarians │ All the powers of old Europe   │
│             │ III. Socialist.. │ have entered into a holy       │
│             │ IV. Position...  │ alliance to exorcise this      │
│             │                  │ spectre...                     │
└─────────────┴──────────────────┴────────────────────────────────┘
```

### Navigation
| Key | Action |
|-----|--------|
| `←` `→` | Move between panels |
| `↑` `↓` | Move within panel |
| `Enter` | Select work or chapter |
| `/` | Activate live search |
| `t` | Cycle color themes |
| `qq` | Quit |

### Session Persistence
The reader remembers position (work + chapter + scroll offset) between sessions using confy. Especially important for Capital Vol. 1.

## Commands

| Command | Description |
|---------|-------------|
| `emarx` | Launch interactive TUI browser |
| `emarx read "Capital 1:4"` | Read specific part:chapter (pipes cleanly to stdout) |
| `emarx search <term>` | Full-text search across all three texts |
| `emarx random` | Display a random passage |
| `emarx today` | Deterministic daily passage (same for all users on a given date) |
| `emarx manifesto` | Jump directly into Manifesto in the TUI |
| `emarx thesis [1-11]` | Print a specific Thesis on Feuerbach by number |
| `emarx specter` | Dramatic letter-by-letter rendering of the Manifesto opening line |
| `emarx intro` | Replay the animated startup banner |
| `emarx contradiction` | Easter egg — prints "All that is solid melts into air." |

### Reference Parsing
Forgiving parser for `read` command:
- `Capital 1:4` → Part 1, Chapter 4
- `capital part1 ch4` → same
- `manifesto 2` → Section 2 of the Manifesto
- `thesis 7` → Thesis VII

## Color Themes

Cycled with `t` in the TUI. Five themes:

| Name | Vibe | Colors |
|------|------|--------|
| **Manifesto** | Stark, urgent | Red foreground, black background |
| **Praxis** | Soviet pamphlet | Muted greens, cream text |
| **Dialectic** | Pure contrast | Black and white only |
| **Vanguard** | Revolutionary luxury | Deep red, gold accents |
| **Capital** | Industrial, cold | Dark grey, steel blue accents |

## Architecture

```
src/
  main.rs          — entry point, CLI dispatch
  cli.rs           — clap command definitions
  app.rs           — TUI app state and event loop
  update.rs        — state update logic (key events → state transitions)
  ui/
    mod.rs         — layout composition
    panels.rs      — works/chapters/text panel rendering
    themes.rs      — color theme definitions
  api/
    mod.rs         — passage lookup, search, random, today logic
  store/
    mod.rs         — session persistence (confy wrapper)
  data/
    mod.rs         — embedded text access (include_str! wrappers)
data/
  manifesto.json
  theses.json
  capital_v1.json
```

## Data Format

Each text is a JSON file with consistent structure:

```json
{
  "title": "The Communist Manifesto",
  "author": "Karl Marx and Friedrich Engels",
  "year": 1848,
  "sections": [
    {
      "id": "preamble",
      "title": "Preamble",
      "paragraphs": ["A spectre is haunting Europe..."]
    }
  ]
}
```

Capital Vol. 1 uses nested `parts` → `chapters` → `paragraphs`.

## Startup Banner

Animated ASCII art banner on launch (tachyonfx), ending with:

```
  emarx — the workers' terminal
  "Workers of all countries, unite!"
```

Replayable with `emarx intro`.

## Error Handling

- Invalid reference → friendly message with example syntax
- Unknown thesis number → list all 11 theses titles
- Search with no results → "No passages found. The dialectic continues."
- Piped output (non-TTY) → plain text, no TUI, no color

## Distribution

- `cargo install emarx` (crates.io)
- `curl -fsSL https://raw.githubusercontent.com/.../install.sh | sh`
- Pre-built binaries for macOS (arm64/x86_64), Linux (x86_64), Windows via GitHub Releases + cargo-dist

## Out of Scope (v1)

- Capital Vol. 2 and Vol. 3
- Online API for additional texts
- Audio or other media
- Non-English translations
- Lenin, Gramsci, or other adjacent authors
