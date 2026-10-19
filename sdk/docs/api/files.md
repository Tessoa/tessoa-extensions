<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# files

The files the user hands to the plugin: the ones selected for the current
command, the ones picked in tessoa's file picker, and new files in a folder
the user picked.

## Functions

### selected

```
wit  selected: func() -> list<file-info>
rust tessoa_plugin_sdk::files::selected() -> Vec<FileInfo>
ts   tessoa.files.selected(): FileInfo[]
```

The files that were selected when the command was triggered.

Permission: none.

### read

```
wit  read: func(handle: handle, offset: u64, max-len: u32) -> result<list<u8>, error>
rust tessoa_plugin_sdk::files::read(handle: Handle, offset: u64, max_len: u32) -> Result<Vec<u8>, Error>
ts   tessoa.files.read(handle: Handle, offset?: number, maxLen?: number): Uint8Array
```

Read up to `max-len` bytes starting at `offset`.
Requires the `files.read-selected` permission; in `columns.values`,
the `files.read-listed` permission.

Permission: `files.read-selected`.

TypeScript also has:

```
tessoa.files.readAll(handle: Handle): Uint8Array
```

### rename

```
wit  rename: func(items: list<rename-request>) -> result<list<rename-outcome>, error>
rust tessoa_plugin_sdk::files::rename(items: &[RenameRequest]) -> Result<Vec<RenameOutcome>, Error>
ts   tessoa.files.rename(items: RenameRequest[]): RenameOutcome[]
```

Ask the host to rename files. One outcome per request, in order; one
failure does not stop the others.
Requires the `files.rename-selected` permission.

Permission: `files.rename-selected`.

### keep

```
wit  keep: func(handle: handle) -> result<file-ref, error>
rust tessoa_plugin_sdk::files::keep(handle: Handle) -> Result<FileRef, Error>
ts   tessoa.files.keep(handle: Handle): FileRef
```

Turn a selected file into a reference that stays valid after the
command returns, and after tessoa restarts. The host follows the file
when tessoa moves or renames it.
Requires the `files.keep` permission.

Permission: `files.keep`.

### pick-files

```
wit  pick-files: func(title: string, extensions: list<string>, multiple: bool) -> result<list<file-ref>, error>
rust tessoa_plugin_sdk::files::pick_files(title: &str, extensions: &[&str], multiple: bool) -> Result<Vec<FileRef>, Error>
ts   tessoa.files.pickFiles(options?: PickFilesOptions): Promise<FileRef[]>
```

Let the user pick files in tessoa's file picker, and wait for the
choice. `extensions` limits the files offered, without the dot, such
as `csv`; empty offers every file. Returns references to the picked
files, read with `refs.read`; empty when the user cancelled. A page
cannot wait, and calls `pick-files-start` instead.

Permission: none.

TypeScript also has:

```
```

### pick-files-start

```
wit  pick-files-start: func(title: string, extensions: list<string>, multiple: bool) -> result<u64, error>
rust tessoa_plugin_sdk::files::pick_files_start(title: &str, extensions: &[&str], multiple: bool) -> Result<u64, Error>
ts   none: await tessoa.files.pickFiles(...) calls it in a page's handler
```

`pick-files` for a page: returns at once with a call id, and the
choice arrives as a `picked-files` event of the page. Only a page can
call it.

Permission: none.

### pick-folder

```
wit  pick-folder: func(title: string) -> result<option<folder>, error>
rust tessoa_plugin_sdk::files::pick_folder(title: &str) -> Result<Option<Folder>, Error>
ts   tessoa.files.pickFolder(options?: PickFolderOptions): Promise<Folder | undefined>
```

Let the user pick a folder, and wait for the choice; none when the
user cancelled. The plugin can create files in the folder with
`create-file`, and nothing else. A page calls `pick-folder-start`.

Permission: none.

TypeScript also has:

```
```

### pick-folder-start

```
wit  pick-folder-start: func(title: string) -> result<u64, error>
rust tessoa_plugin_sdk::files::pick_folder_start(title: &str) -> Result<u64, Error>
ts   none: await tessoa.files.pickFolder(...) calls it in a page's handler
```

`pick-folder` for a page: the choice arrives as a `picked-folder`
event. Only a page can call it.

Permission: none.

### folder-name

```
wit  folder-name: func(folder: folder) -> result<string, error>
rust tessoa_plugin_sdk::files::folder_name(folder: Folder) -> Result<String, Error>
ts   tessoa.files.folderName(folder: Folder): string
```

The name of a picked folder, without the directories above it.

Permission: none.

### release-folder

```
wit  release-folder: func(folder: folder)
rust tessoa_plugin_sdk::files::release_folder(folder: Folder)
ts   tessoa.files.releaseFolder(folder: Folder): void
```

Forget a picked folder; creating files in it fails afterwards.

Permission: none.

### create-file

```
wit  create-file: func(folder: folder, path: string, data: list<u8>) -> result<create-outcome, error>
rust tessoa_plugin_sdk::files::create_file(folder: Folder, path: &str, data: &[u8]) -> Result<CreateOutcome, Error>
ts   tessoa.files.createFile(folder: Folder, path: string, data: Uint8Array | string): Promise<CreateOutcome>
```

Create a file in a picked folder, and wait until it is written.
`path` is relative to the folder and written with `/`, such as
`report.csv` or `2026/report.csv`; missing folders on the way are
created. A path with `..`, a drive, a backslash, a name Windows
reserves (such as `con`) or a name that ends in a dot or a space is
refused as `invalid`. At most 64 MiB. An existing file is never
replaced: when the name is taken, tessoa asks the user whether to keep
both, with a number added to the new name, or to skip it. The new file
is in tessoa's undo and activity log. A page calls
`create-file-start`.
Requires the `files.create` permission.

Permission: `files.create`.

### create-file-start

```
wit  create-file-start: func(folder: folder, path: string, data: list<u8>) -> result<u64, error>
rust tessoa_plugin_sdk::files::create_file_start(folder: Folder, path: &str, data: &[u8]) -> Result<u64, Error>
ts   none: await tessoa.files.createFile(...) calls it in a page's handler
```

`create-file` for a page: returns at once with a call id, and the
outcome arrives as a `created` event of the page. Only a page can
call it.
Requires the `files.create` permission.

Permission: `files.create`.

## Types

### handle

`handle` is `u32` (Rust `u32`, TypeScript `number`).

Identifies one selected file for the duration of the current command.
Handles are issued by the host when the command starts and are invalid
after it returns.

### file-ref

`file-ref` is `u64` (Rust `u64`, TypeScript `number`).

Identifies a file the host keeps for the plugin, across commands and
across restarts. References come from the user: a file kept with
`keep`, a file dropped on a page, a file picked with `pick-files`, or
a file the plugin created with `create-file`. The plugin sees the
number only.

### folder

`folder` is `u64` (Rust `u64`, TypeScript `number`).

Identifies a folder the user picked for the plugin to create files
in. It is valid until the plugin stops or tessoa restarts, or until
`release-folder`; the plugin sees its name, never its path.

### file-info

Record; Rust `FileInfo`, TypeScript `FileInfo`.

| Field | Type | Meaning |
|---|---|---|
| `handle` | `handle` |  |
| `name` | `string` | File name without the directory. Plugins never see the directory. |
| `size` | `u64` | Size in bytes. |
| `modified-ms` | `s64` | Last modification time as Unix epoch milliseconds. |
| `is-dir` | `bool` |  |

### error

Variant; Rust `Error`, TypeScript `FilesError` (thrown) with `kind: ErrorKind`.

| Case | Payload | Meaning |
|---|---|---|
| `not-granted` | `string` | The handle or reference was not issued to this plugin, or is no longer valid. |
| `permission-denied` | `string` | The manifest does not declare the permission this call needs. |
| `io` | `string` | The file system or the host reported an error. |
| `invalid` | `string` | The request cannot be carried out as made, for example a path with `..` in it, or a call a page cannot make. Carries the reason. |

### rename-request

Record; Rust `RenameRequest`, TypeScript `RenameRequest`.

| Field | Type | Meaning |
|---|---|---|
| `handle` | `handle` |  |
| `new-name` | `string` | New file name without the directory. Names that contain a path separator or `..` are rejected. |

### rename-outcome

Variant; Rust `RenameOutcome`, TypeScript `RenameOutcome`.

| Case | Payload | Meaning |
|---|---|---|
| `renamed` | `string` | Renamed. Carries the final name; the host may have changed it to resolve a conflict. |
| `skipped` | `string` | Nothing to do, for example because the name did not change. Carries the reason. |
| `failed` | `string` | Failed. Carries the reason. |

### create-outcome

Variant; Rust `CreateOutcome`, TypeScript `CreateOutcome`.

What `create-file` did.

| Case | Payload | Meaning |
|---|---|---|
| `created` | `file-ref` | Created. Carries a reference to the new file, whose name has a number added when the user chose to keep both. |
| `skipped` | `string` | Not created: the user chose to skip it because a file of that name exists. Carries the reason. |
