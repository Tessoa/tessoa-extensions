# Changes

What changed in each release of the SDK kit, newest first. The kit version is
the one in `sdk/VERSION`; the interface version is the `api` a plugin names in
`plugin.toml`. `tessoa-plugin upgrade` prints the sections between the kit it
replaces and the one it installs.

Each release lists its changes in up to three groups: Interface (what a plugin
can call or export), Tool (the `tessoa-plugin` command-line tool and the
runner it tests with) and Kit (the manual, the guides, the reference and the
examples).

## 0.2.0 (2026-11-10)

### Interface

- `pages-host.followed(page)`: a page declared with `follow = true` in
  `[[pages]]` shows what the user selects in one tab group, the way tessoa's
  own preview tabs do. tessoa draws the chip above the page that names what it
  follows, and the user changes it there. TypeScript pages get
  `follow(state, followed)`; Rust pages get `Page::follow` behind the SDK's
  `follow` feature. The function needs no permission, and the interface stays
  `tessoa:plugin@0.2.0`: it only grew. A tessoa before 0.30.0 opens such a page
  as a plain one.

### Tool

- Scenarios can play tessoa telling a page what it follows:
  `follow = { group = 2, file = "b.json" }`, told again after `restart`.
- A list without rows shows its `empty` text to `expect-text`.
- A list that gets shorter while its last rows are on screen no longer shows
  empty rows for a frame.
- `--json` on every command: stdout carries one JSON document at the end
  with the result and the problems, and what is printed for a person goes to
  stderr. Each problem has a stable code and a hint on how to fix it, also
  printed as a `hint:` line without `--json`. The codes are listed in
  `docs/guide/cli.md`.
- Exit statuses are the same in every command: 1 when the plugin, its build,
  its tests or the kit fail, 2 when the command cannot start (wrong
  arguments, no project, `skeleton/` left, a missing tool or tessoa). `doctor`
  now exits 2 for a missing toolchain, and a folder that is not a project
  exits 2 everywhere, where it was 1 before.
- `new <folder> --lang ts|rust --id <id>` creates a project without the
  template: the skeleton carried by the tool with the id and name filled in,
  the manual and its pointer files, the skill for coding tools, and the kit in
  `sdk/`, downloaded through the signed manifest `upgrade` reads, or taken
  from `--kit`.
- `dev` builds, audits and installs the plugin, then builds and audits again
  after every change to its sources, with `--test` running the scenarios too,
  until Ctrl+C.
- `skill install` writes the skill for coding tools into the user's own
  skills, or with `--project` into the project; `--check` only reports.
- `upgrade` prints the sections of `sdk/CHANGELOG.md` between the kit it
  replaces and the one it installs.
- `mcp` serves the commands to a coding tool as an MCP server on stdin and
  stdout: `new_project`, `doctor`, `check`, `build`, `test`, `render`,
  `package` and `install` return the `--json` document as their result, and
  `api_reference`, `manifest_reference`, `guide`, `list_examples` and
  `example_source` serve the reference, the guides and the examples built
  into the tool, which are offered as resources as well.
- `install` and `dev` on Windows no longer keep a caller waiting that reads
  their output through a pipe when they start tessoa: the tessoa they start
  gets none of their handles and is not stopped by their Ctrl+C.
- When `TESSOA_EXE` is set, `install` uses only that path and reports a
  wrong one, instead of handing the plugin to another tessoa on the PATH.

### Kit

- The JSON viewer and the CSV table examples follow the selection.
- This file, `CHANGELOG.md`, is part of the kit.
- `docs/guide/ai-tools.md`: where a coding tool finds the manual, before and
  after the project exists.
- A skill for coding tools, made from the manual word for word behind a
  short preface. The template carries it; `new` and `skill install` write it.
- `llms.txt` and `llms-full.txt`, an index and the full text of the manual,
  the guides, the API reference and the examples, are generated with each
  release, to be published under tessoa.cn/plugins/.

## 0.1.0 (2026-10-20)

The first release.

### Interface

- `tessoa:plugin@0.2.0`, frozen: plugins built for it keep working with every
  later runner. Commands on the selected files, pages (a tab or a sidebar
  section that tessoa draws with its own widgets), columns in tessoa's file
  list, settings tessoa shows and keeps, tasks for long work, connections to
  the websites a plugin lists, storage, the clipboard, images, dialogs and
  files the user picks or creates.

### Tool

- `tessoa-plugin` with `init`, `upgrade`, `doctor`, `check`, `build`, `test`,
  `render`, `run`, `package` and `install`. `test` runs scenarios against the
  plugin in the real sandbox; page scenarios play the user step by step.
  `install` hands the project to the running tessoa, which asks the user to
  confirm the permissions.
- `upgrade` updates the tool itself and replaces `sdk/` from a signed manifest.

### Kit

- The manual `AGENTS.md`, the guides, the API reference generated from the
  interface, the interface definition, the TypeScript declarations with the
  JSX runtime, and the Rust SDK.
- Examples in TypeScript and in Rust, each a project of its own under the MIT
  licence.
