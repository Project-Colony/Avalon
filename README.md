<div align="center">

# Avalon

**A desktop writing studio for novels, scripts and other long-form projects.**

</div>

[![License: GPL-3.0-or-later](https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg)](LICENSE)

Avalon organizes a manuscript as a binder of documents and folders, with an
editor, notes, snapshots, writing targets and a compiler that exports the
whole project. It is written in Rust with the [iced](https://iced.rs) GUI
toolkit. Its workflow is inspired by Scrivener; Avalon is an independent
project and is not affiliated with Literature & Latte.

> **Status:** early prototype. It has never been released and has no binary
> builds yet. Development paused in February 2026. Import of real `.scriv`
> projects is untested, and the code still uses its working name "Scrinever"
> in places (for example the `~/.config/scrinever` folder).

## What it does

- Binder with documents, folders, labels, status and keywords
- Markdown editor with find and replace, spell check, thesaurus and name generator
- Snapshots, automatic backups and session or project word targets
- Compile and export to PDF, DOCX, EPUB, HTML, LaTeX, RTF, Fountain, Markdown, OPML and plain text
- Import from `.scriv`, `.docx`, RTF, HTML and Markdown

## Projects and files

A project is a folder holding `project.json`, a `docs/` folder with the text of
each document and a `snapshots/` folder. **Open** asks for that folder. The
first **Save** of a new project asks where to create it, starting in
`~/Scrinever Projects`. From then on, saving, autosave and settings changes
write to that same folder, even after you rename the project. Autosave starts
once a project has been saved for the first time.

**Import** asks for the files to add to the draft. For a Scrivener project,
pick the `.scrivx` file inside its `.scriv` folder. **Compile** and **Export
OPML** ask where to write the result. These dialogs start in the folder you
used last during the session. On Linux they come from the XDG desktop portal,
or from `zenity` when no portal is running.

## Build from source

```bash
git clone https://github.com/Project-Colony/Avalon
cd Avalon
cargo build --release
```

Requires a recent stable Rust toolchain. On Debian or Ubuntu, install
`libfontconfig1-dev` and `pkg-config` first.

## Privacy

Avalon makes no network connections and collects no telemetry. It reads and
writes only your project files, the files you choose to import or export, its
configuration and backup folders, and the system clipboard when you copy or
paste.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

The bundled JetBrains Mono Nerd Font files in `assets/fonts/` are under the
SIL Open Font License 1.1; see [assets/fonts/OFL.txt](assets/fonts/OFL.txt).
