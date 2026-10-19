<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# refs

Files the host keeps for the plugin. Every function works on references
issued to this plugin; any other reference is refused.

## Functions

### info

```
wit  info: func(refs: list<file-ref>) -> list<ref-info>
rust tessoa_plugin_sdk::refs::info(refs: &[FileRef]) -> Vec<RefInfo>
ts   tessoa.refs.info(refs: FileRef[]): RefInfo[]
```

Describe references. References that were never issued to this plugin,
or were released, are left out of the result.

Permission: none.

### read

```
wit  read: func(ref: file-ref, offset: u64, max-len: u32) -> result<list<u8>, error>
rust tessoa_plugin_sdk::refs::read(ref: FileRef, offset: u64, max_len: u32) -> Result<Vec<u8>, Error>
ts   tessoa.refs.read(ref: FileRef, offset?: number, maxLen?: number): Uint8Array
```

Read up to `max-len` bytes of a referenced file, starting at `offset`:
a file the user dropped on a page or picked, or one the plugin kept or
created.
Requires the `files.read-selected` permission.

Permission: `files.read-selected`.

TypeScript also has:

```
tessoa.refs.readAll(ref: FileRef): Uint8Array
```

### release

```
wit  release: func(refs: list<file-ref>)
rust tessoa_plugin_sdk::refs::release(refs: &[FileRef])
ts   tessoa.refs.release(refs: FileRef[]): void
```

Forget references. The files themselves are not touched.

Permission: none.

### copy-to-picked

```
wit  copy-to-picked: func(refs: list<file-ref>) -> result<u64, error>
rust tessoa_plugin_sdk::refs::copy_to_picked(refs: &[FileRef]) -> Result<u64, Error>
ts   tessoa.refs.copyToPicked(refs: FileRef[]): number
```

Let the user pick a folder and copy the files there, with tessoa's
progress, conflict handling and undo. Returns at once with a transfer
id; the outcome arrives as a `transfer-done` event of the page that
made the call.
Requires the `files.transfer` permission.

Permission: `files.transfer`.

### move-to-picked

```
wit  move-to-picked: func(refs: list<file-ref>) -> result<u64, error>
rust tessoa_plugin_sdk::refs::move_to_picked(refs: &[FileRef]) -> Result<u64, Error>
ts   tessoa.refs.moveToPicked(refs: FileRef[]): number
```

Like `copy-to-picked`, moving instead of copying. The references
follow the files.
Requires the `files.transfer` permission.

Permission: `files.transfer`.

### open

```
wit  open: func(ref: file-ref) -> result<_, error>
rust tessoa_plugin_sdk::refs::open(ref: FileRef) -> Result<(), Error>
ts   tessoa.refs.open(ref: FileRef): void
```

Open the file the way a double click in tessoa would.
Requires the `files.open` permission.

Permission: `files.open`.

### reveal

```
wit  reveal: func(ref: file-ref) -> result<_, error>
rust tessoa_plugin_sdk::refs::reveal(ref: FileRef) -> Result<(), Error>
ts   tessoa.refs.reveal(ref: FileRef): void
```

Show the file in a new tessoa tab, selected.
Requires the `files.open` permission.

Permission: `files.open`.

### trash

```
wit  trash: func(refs: list<file-ref>) -> result<u64, error>
rust tessoa_plugin_sdk::refs::trash(refs: &[FileRef]) -> Result<u64, Error>
ts   tessoa.refs.trash(refs: FileRef[]): number
```

Move the files to the recycle bin (the Trash on macOS). tessoa always
asks the user first, in its own dialog that names the plugin; the
plugin cannot skip that. Returns at once with a transfer id; the
outcome arrives as a `transfer-done` event of the page that made the
call.
Requires the `files.delete` permission.

Permission: `files.delete`.

## Types

### file-ref

Defined in [files](files.md#file-ref).

### error

Defined in [files](files.md#error).

### ref-info

Record; Rust `RefInfo`, TypeScript `RefInfo`.

| Field | Type | Meaning |
|---|---|---|
| `ref` | `file-ref` |  |
| `name` | `string` | File name without the directory. |
| `size` | `u64` | Size in bytes; 0 for a folder or a file that is gone. |
| `modified-ms` | `s64` | Last modification time as Unix epoch milliseconds. |
| `is-dir` | `bool` |  |
| `exists` | `bool` | False when the file is no longer where the host last saw it. |

### transfer-outcome

Variant; Rust `TransferOutcome`, TypeScript `TransferOutcome`.

What happened to one file of a copy, a move or a delete.

| Case | Payload | Meaning |
|---|---|---|
| `done` | none | Copied, moved, or moved to the recycle bin. |
| `skipped` | `string` | Left alone, for example because the user cancelled. Carries the reason. |
| `failed` | `string` | Failed. Carries the reason. |
