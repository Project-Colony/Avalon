# Security Policy

Avalon is a desktop writing studio. It makes no network connections and runs
no privileged code, so its attack surface is the files it reads: documents you
import and project folders you open.

## Supported versions

Avalon has not been released yet. Security fixes land on `main`; once releases
exist, only the latest one will receive fixes.

## Reporting a vulnerability

Please report vulnerabilities **privately** through GitHub:

<https://github.com/Project-Colony/Avalon/security/advisories/new>

Do not open a public issue for an exploitable bug. Include what an attacker
controls, what they get, the commit you tested, your operating system, and a
sample file or steps that reproduce it.

You can expect an acknowledgement within a few days, a fix or mitigation plan
before any public disclosure, coordinated with you, and credit in the release
notes if you want it.

## Scope

Avalon treats every file below as untrusted input. Reports of particular
interest:

- **Imports**: Scrivener `.scriv` packages (the `.scrivx` XML and the RTF or
  text files it points to), `.docx` archives, RTF, HTML, Markdown, LaTeX,
  Fountain and OPML files. A file that makes Avalon read or write outside the
  package or the project, or exhaust memory or CPU (for example a zip bomb), is
  in scope.
- **Project folders**: `project.json` and the files under `docs/` and
  `snapshots/` of a project you open, including one listed in the recent
  projects. Avalon saves a project only into the folder it was opened from or
  the one chosen in the save dialog, and a project title never changes where
  files go; a way around that is in scope.
- **Backup archives**: the `.zip` files in the backup folder that **Restore as
  copy** unpacks. A restore writes only `project.json`, `compile_presets.json`
  and `docs/<document id>.json` into a new folder next to the project; an
  archive that writes anywhere else, or exhausts memory or disk, is in scope.

Out of scope: attacks that need someone who can already run code as you or
edit Avalon's own configuration, and bugs in the operating system or in the
libraries Avalon depends on (report those upstream).
