<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# columns

Values for the columns `[[columns]]` in `plugin.toml` declares, which the
user can add to tessoa's file list. tessoa asks only for the files on
screen, at most 64 at a time, keeps the answers, and asks again when a
file changes. The values are computed in a worker, as for a task. Every
plugin exports it; a plugin without columns answers with empty values.

Every plugin exports this interface. The runner calls it in a worker, for the files of tessoa's file list that are on screen. In Rust, implement `tessoa_plugin_sdk::Columns` and name the type in `tessoa_plugin_sdk::export!` or `export_pages!`, as `columns: Dims`; the SDK asks it for one file at a time. In TypeScript, call `tessoa.columns.define(column, value)` for each column when the script loads. A plugin without columns answers with empty values.

## Functions

### values

```
wit  values: func(column: string, files: list<column-file>) -> list<column-value>
rust fn value(&mut self, column: &str, file: &ColumnFile) -> ColumnValue   (in impl Columns, named in export!)
ts   tessoa.columns.define(column, async (file) => value)
```

The values of `column` for `files`: one per file, in order. A list
of another length counts as empty values.

## Types

### handle

Defined in [files](files.md#handle).

### column-file

Record; Rust `ColumnFile`, TypeScript `ColumnFile`.

One file of the file list. The handle is valid during this call
only; reading the file needs the `files.read-listed` permission.

| Field | Type | Meaning |
|---|---|---|
| `handle` | `handle` |  |
| `name` | `string` | File name without the directory. |
| `size` | `u64` | Size in bytes; 0 for a folder. |
| `modified-ms` | `s64` | Last modification time as Unix epoch milliseconds. |
| `is-dir` | `bool` |  |

### column-value

Variant; Rust `ColumnValue`, TypeScript `ColumnValue`.

The value of one cell. Numbers, sizes and dates are written by
tessoa, the way its own columns write them.

| Case | Payload | Meaning |
|---|---|---|
| `empty` | none | Nothing to show; sorts last. |
| `text` | `string` | Text; sorts the way tessoa sorts names. |
| `number` | `f64` |  |
| `size` | `u64` | A size in bytes. |
| `date` | `s64` | A time as Unix epoch milliseconds. |
| `sorted-text` | `tuple<string, f64>` | Text that sorts by a number, such as `1920×1080` by the pixel count. |
