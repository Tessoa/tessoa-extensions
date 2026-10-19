# Files

A plugin never sees a path and has no file system of its own. Every file it
works with is one the user handed to it, and every change is a request that
tessoa carries out with its own conflict handling, undo and activity log.
This page lists the ways files reach a plugin, and covers the two that let a
plugin ask: picking files, and creating files in a folder the user picks.

| How the file comes | What the plugin gets | Valid | Reading it | See |
|---|---|---|---|---|
| Selected for a command | a handle, `files.selected()` | during the command, and in a task it starts | `files.read`, with `files.read-selected` | [manifest.md](manifest.md) |
| Kept by a command, `files.keep(handle)` | a file reference | across restarts, until released | `refs.read`, with `files.read-selected` | [pages.md](pages.md#files) |
| Dropped on a page's drop zone | file references | across restarts, until released | `refs.read`, with `files.read-selected` | [pages.md](pages.md#files) |
| Picked in tessoa's file picker | file references | across restarts, until released | `refs.read`, with `files.read-selected` | below |
| Created in a folder the user picked | a file reference | across restarts, until released | `refs.read`, with `files.read-selected` | below |
| Shown in tessoa's file list, for a column | a handle | during the request | `files.read`, with `files.read-listed` | [columns.md](columns.md) |

A file reference follows the file when tessoa moves or renames it. Keep the
numbers in the storage or the page's saved state to remember files, and
release the ones the plugin forgets.

## Picking files

```ts
const files = await tessoa.files.pickFiles({ title: "Open a CSV file", extensions: ["csv"], multiple: false });
if (files.length > 0) {
  const text = new TextDecoder().decode(tessoa.refs.readAll(files[0]));
}
```

tessoa shows its own file picker with the title, followed by the plugin's
name. `extensions` (without the dot) limits the files offered; without it
every file shows. With `multiple` the user may pick several. The promise
resolves to references to the picked files, empty when the user cancelled.
Reading them needs `files.read-selected`: the user handed them over, as with
a selection.

`pickFiles` is the same in a page's handler, a command and a task: in a
handler the page keeps working while the picker is open. In Rust a command
or a task waits for the choice, and a page asks and hears it later:

```rust
// In a command or a task:
let files = files::pick_files("Open a CSV file", &["csv"], false).map_err(|e| format!("{e:?}"))?;

// In a page:
self.picking = Some(files::pick_files_start("Open a CSV file", &["csv"], false).map_err(|e| format!("{e:?}"))?);
// ...and in event:
Event::PickedFiles { id, files } if Some(id) == self.picking => self.open(files),
```

`examples/csv-table-ts` and `examples/json-tree-ts` open the file they show
this way.

## Creating files

A plugin cannot write anywhere it likes. It asks the user for a folder, then
creates new files in it; it can never change or delete a file that exists.

```ts
const folder = await tessoa.files.pickFolder({ title: "Export to" });
if (folder !== undefined) {
  const outcome = await tessoa.files.createFile(folder, "report.csv", csvText);
  if ("created" in outcome) tessoa.refs.reveal(outcome.created); // reveal needs files.open
}
```

- `pickFolder` shows tessoa's folder picker and resolves to the folder,
  `undefined` when the user cancelled. The plugin sees its name
  (`folderName(folder)`), never its path, and can create files in it until
  the plugin stops, tessoa restarts or `releaseFolder(folder)`; then it asks
  again. The folder number is not a file reference and does not survive a
  restart.
- `createFile(folder, path, data)` writes a new file and resolves once it is
  written. `path` is relative to the folder and written with `/`, such as
  `report.csv` or `2026/report.csv`; missing folders on the way are created.
  A path with `..`, a drive, a backslash, a name Windows reserves (such as
  `con`) or a name that ends in a dot or a space is refused as `invalid`.
  `data` is bytes or text (written as UTF-8), at most 64 MiB.
- **A name that is taken** is never replaced: tessoa asks the user whether
  to keep both, the new file numbered (`report (2).csv`), or to skip it. The
  outcome is `{ created: ref }`, a reference to the new file whatever its
  final name, or `{ skipped: why }`.
- The new file is in tessoa's undo and activity log under the plugin's name.
- Creating needs the `files.create` permission; picking a folder needs none.

In Rust, `files::pick_folder` and `files::create_file` wait in a command or a
task; a page calls `pick_folder_start` and `create_file_start`, and hears
`Event::PickedFolder { id, folder }` and `Event::Created { id, outcome }`.
`CreateOutcome::Created(file)` and `CreateOutcome::Skipped(why)` are the two
outcomes.

Exporting is a good use for a task ([tasks.md](tasks.md)): the task picks the
folder, writes the files with its progress, and the user can cancel it. A
page's handler that awaits a task's end cannot pick a folder afterwards.
`examples/hash-files-rs` saves its report this way.

## What the plugin cannot do

- Read a file the user did not hand to it, or list a folder.
- Change, overwrite or delete a file, except by asking tessoa: renaming the
  selected files in a command (`files.rename`), moving references to the
  recycle bin after tessoa asks the user (`refs.trash`), copying or moving
  references to a folder the user picks (`refs.copyToPicked`,
  `refs.moveToPicked`). See [permissions.md](permissions.md).
- Remember a picked folder across restarts.

## Testing

A scenario answers the pickers and the name conflicts in the step whose
action leads to them: `pick-files = ["a.csv"]` picks files of the scenario's
folder (`[]` cancels), `pick-folder = "out"` picks a folder in it (`false`
cancels), `answer-conflict = "keep-both"` or `"skip"` answers a name that is
taken. `expect-created` checks the files the plugin created. A page's picker
may be answered by a later step; a command's and a task's must be answered in
the same step. See [testing.md](testing.md#answers).

```toml
[[steps]]
click = "export"
pick-folder = "out"
expect-created = ["out/report.csv"]
```
