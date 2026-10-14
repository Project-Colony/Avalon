<div align="center">

# Avalon

**A desktop writing studio for novels, scripts and other long-form projects.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-office-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

Scrivener is the usual tool for writing a book in pieces, but it is proprietary,
paid, and has no Linux version; a word processor keeps the whole manuscript in
one long file instead. Avalon is a free writing studio for Linux, Windows and
macOS that keeps each chapter or scene as its own document in a binder, with
notes, snapshots, word targets and a compiler that exports the whole project
in one step.

> **Status:** early, and not released yet. Writing, organising the binder,
> saving, snapshots, backups, import and compile all work and have unit tests,
> and CI builds and tests Avalon on Linux, Windows and macOS. Not checked yet:
> importing real Scrivener projects (the importer is only tested on small
> hand-made ones) and how the interface looks on Windows and macOS. Spell check
> knows 4,812 English words plus the ones you add, so it flags many correct
> words, and the thesaurus is a small list of common English words. Some
> folders still carry the working name Scrinever (see [Privacy](#privacy)).
> Until the first signed release is published, build from source.

## Why Avalon

- **Runs on Linux too.** Scrivener runs on macOS, Windows and iOS. Avalon is
  one native program for Linux, Windows and macOS.
- **A binder, not one long file.** Each chapter, scene or note is its own
  document in a tree of folders. You give it a label, a status and keywords,
  and write it without scrolling past the rest of the book.
- **Plain project files.** A project is an ordinary folder of JSON files that
  you can read, sync and keep under version control. There is no account and no
  cloud service.
- **Export without extra tools.** Compile writes PDF, DOCX, EPUB and seven
  other formats by itself, with no Pandoc or LaTeX installation needed.

Avalon's workflow is inspired by Scrivener. It is an independent project and is
not affiliated with Literature & Latte.

## What it does

- Binder with documents, folders, labels, status and keywords
- Markdown editor with find and replace, spell check with a personal dictionary,
  a small built-in thesaurus of common English words and a name generator
- Per-project editor font, size, zoom, line spacing and text width, and smart
  punctuation as you type, on by default: curly quotes, em dashes and ellipses.
  A Backspace right after a correction puts back what you typed
- Snapshots (`F9`), automatic backups and session or project word targets
- Compile and export to PDF, DOCX, EPUB, HTML, LaTeX, RTF, Fountain, Markdown,
  OPML and plain text
- Import from Scrivener (`.scriv`), DOCX, RTF, HTML, Markdown, LaTeX, Fountain,
  OPML and plain text

### Projects and files

A project is a folder holding `project.json`, a `docs/` folder with the text of
each document and a `snapshots/` folder. **Open** asks for that folder. The
first **Save** of a new project asks where to create it, starting in
`~/Scrinever Projects`. From then on, saving, autosave and settings changes
write to that same folder, even after you rename the project. Autosave starts
once a project has been saved for the first time. A setting is saved the moment
you change it, with the Settings window still open; a project that was never
saved keeps its settings until its first save.

**Backups** are zip archives of `project.json`, the `docs/` folder and the
compile presets, written to `~/Scrinever Backups` as
`<project folder>_<UTC timestamp>.zip`. Snapshots are left out to keep them
small. With automatic backups on (Settings, Backup), a backup is made every 10
saves by default; manual saves and autosaves both count, and the interval is a
project setting. **Create Backup** in the Backups panel (`Ctrl+8`), or
`Ctrl+Shift+B`, makes one at once. The 20 newest backups of each project are
kept. **Restore as copy** unpacks a backup into a new folder next to the
project, named `<project> restored <timestamp>`, and opens it; the project
folder itself is never overwritten.

**Import** asks for the files to add to the draft. For a Scrivener project,
pick the `.scrivx` file inside its `.scriv` folder. RTF files come in as plain
text: formatting, images and headers are dropped, and table cells are separated
by tabs. **Compile** and **Export OPML** ask where to write the result. These
dialogs start in the folder you used last during the session. On Linux they
come from the XDG desktop portal, or from `zenity` when no portal is running.

**Print** and **Print All** write a PDF to `~/Scrinever Projects`, as
`print.pdf` for the current document and `<project>_print.pdf` for the whole
project, for you to open and print.

## Installation

### Via Colony (recommended)

Search for **Avalon** in [Colony](https://github.com/Project-Colony/Colony) and
install it. Updates arrive through the launcher.

### Direct binary download

Grab the asset for your platform from the
[latest release](../../releases/latest):

| Platform | Asset |
|---|---|
| Linux | `avalon-linux` |
| Windows | `avalon-windows.exe` |
| macOS (Apple Silicon) | `avalon-macos` |
| macOS (Intel) | `avalon-macos-x86` |

```bash
chmod +x avalon-linux && ./avalon-linux
```

Run it with `--version` (or `-V`) to print the version, for example
`avalon 0.1.0`, without opening a window.

### Build from source

```bash
git clone https://github.com/Project-Colony/Avalon
cd Avalon
cargo build --release
```

Requires Rust 1.91 or newer. On Debian or Ubuntu, install `libfontconfig1-dev`
and `pkg-config` first. Avalon is written in Rust with the
[iced](https://iced.rs) GUI toolkit.

## Code signing policy

Every release asset is signed by the Project Colony organisation in CI, never on
a developer machine. Next to each asset on the release page:

| File | What it is |
|---|---|
| `<asset>.sig` | an ed25519 signature over the asset, made with the organisation's release key |
| `<asset>.meta` | three lines binding the asset to its file name, its sha256 and the release version |
| `<asset>.meta.sig` | an ed25519 signature over the `.meta` |

The private key is an organisation secret, used only by the shared
[sign-and-publish workflow](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/.github/workflows/sign-and-publish.yml)
in a job that builds nothing; the jobs that compile Avalon never see it.
Colony checks all three files before it installs or updates Avalon, and
refuses a release older than the one installed. To check a download yourself
with OpenSSL 3 (the same commands work for every asset):

```bash
cat > colony-release.pub <<'EOF'
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEARNjg3Nn8H6/aBg1unwGjkUTcrdTxERNefVaqU8cFu0s=
-----END PUBLIC KEY-----
EOF
a=avalon-linux
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a" -sigfile "$a.sig"
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a.meta" -sigfile "$a.meta.sig"
cat "$a.meta"     # version=<tag>, asset=<file name>, sha256=<digest>
sha256sum "$a"    # the digest must equal the sha256 line
```

How releases are built, signed and published:
[design/releases.md](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/design/releases.md#5-signing).

## Privacy

Avalon sends no telemetry, no analytics and no crash reports. It has no account
and stores no credentials.

`~` below is your home folder (`%USERPROFILE%` on Windows); Avalon uses the
same paths on every platform.

| Data | Stored or sent | Where, and why |
|---|---|---|
| Projects | stored | the folder you choose on the first **Save**, suggested inside `~/Scrinever Projects`; holds the text, notes, snapshots and the project's settings (editor font and layout, smart punctuation, targets, backup interval) |
| Backups | stored | `~/Scrinever Backups`, zip archives of your projects, made automatically every few saves when automatic backups are on, or when you ask for one |
| Recent projects | stored | `~/.config/scrinever/recent.json`: the title, folder and last-opened time of the 20 projects you opened last, for the welcome screen |
| Personal dictionary | stored | `~/.avalon/user_dictionary.txt`: the words you add to spell check |
| Print output | stored | `~/Scrinever Projects/print.pdf` and `<project>_print.pdf`, written when you use **Print** or **Print All** |
| Imports and exports | read or stored | only the files you pick in an Import, Compile or Export dialog |
| Clipboard | read and written | when you copy or paste in the editor, or use the copy-as-Markdown, HTML or plain-text buttons |

Avalon connects to no server. This program will not transfer any information to
other networked systems unless specifically requested by the user or the person
installing or operating it.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

The bundled JetBrains Mono Nerd Font files in `assets/fonts/` are under the
SIL Open Font License 1.1; see [assets/fonts/OFL.txt](assets/fonts/OFL.txt).
