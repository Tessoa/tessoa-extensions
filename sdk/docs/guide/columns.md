# Columns

A plugin can add columns to tessoa's file list: the size of a picture, the
number of lines of a text file, the title of a document. The user adds a
column from the list's column menu, where the columns of the plugins come
after tessoa's own, grouped by plugin; none is shown until the user adds it.
tessoa asks the plugin only for the files on screen, keeps the answers, and
writes numbers, sizes and dates the way its own columns do.

`examples/image-size-column-rs` is a complete one: the width and height of
PNG and JPEG images, read from the first bytes of each file.

## Declare the column

```toml
[[columns]]
id = "lines"
title = "Lines"
title-zh = "行数"
width = 6
align = "end"
when = { ext = ["txt", "md", "csv"] }

[permissions]
files = ["read-listed"]
```

| Key | Meaning |
|---|---|
| `id` | Names the column in the plugin's code; unique within the plugin, not empty. |
| `title` | The header, and the name in the column menu; `title-zh`, `title-ja`, `title-de` for other languages. |
| `width` | Where its width starts, in em, from 1 to 100; default 8. The user resizes it. |
| `align` | `start` (the default), `center` or `end`. |
| `sortable` | Default `true`: a click on the header sorts the list by the column. |
| `when` | Only for these files: `ext` (extensions without the dot; empty or absent means any file) and `dirs` (`true` gives folders a value too; default `false`). The others show nothing and cost nothing. |

A plugin may have columns only, without commands or pages. A plugin with
columns is shown to the user with a line saying it sees the names of the
files in the folders they open, for its columns.

## Give the values

TypeScript: define each column when the script loads.

```ts
tessoa.columns.define("lines", (file) => {
  const text = new TextDecoder().decode(tessoa.files.readAll(file.handle));
  return text === "" ? 0 : text.split("\n").length - (text.endsWith("\n") ? 1 : 0);
});
```

Rust: a type that implements `Columns`, listed in the export.

```rust
struct Lines;

impl Columns for Lines {
    fn value(&mut self, column: &str, file: &ColumnFile) -> ColumnValue {
        match (column, files::read_all(file.handle)) {
            ("lines", Ok(bytes)) => {
                let breaks = bytes.iter().filter(|&&b| b == b'\n').count();
                let unended = bytes.last().is_some_and(|&b| b != b'\n');
                ColumnValue::Number((breaks + usize::from(unended)) as f64)
            }
            _ => ColumnValue::Empty,
        }
    }
}

tessoa_plugin_sdk::export!(MyPlugin; columns: Lines);
```

The function receives one file of the list: `handle`, `name`, `size`,
`modifiedMs` (Rust `modified_ms`) and `isDir` (`is_dir`). The plugin never
sees where the file is. The handle is valid while the function runs; reading
the file through it needs `files.read-listed` (`files.read-selected` does not
count here, nor the other way round).

| A value | TypeScript returns | Rust | Shows, and sorts |
|---|---|---|---|
| Text | `"Report"`, or `{ text }` | `ColumnValue::text(s)` | as is; sorts the way tessoa sorts names |
| A number | `12`, or `{ number }` | `ColumnValue::Number(n)` | written by tessoa; sorts by the number |
| A size in bytes | `{ size: 1024 }` | `ColumnValue::Size(n)` | as tessoa writes sizes |
| A time | a `Date`, or `{ date: ms }` | `ColumnValue::Date(ms)` | as tessoa writes dates |
| Text that sorts by a number | `{ sortedText: ["1920×1080", 2073600] }` | `ColumnValue::sorted_text(s, n)` | the text; sorts by the number |
| Nothing | `null` or `undefined` | `ColumnValue::Empty` | an empty cell; sorts last |

Text longer than 1 KiB is cut, and a number that is not finite counts as
nothing. In TypeScript a file whose function throws shows nothing, and the
first error of each request goes to the plugin log; the function may be
`async`.

## How tessoa asks

- **Only for what shows.** tessoa asks for the files on screen that it has no
  value for, at most 64 at a time and one request at a time per plugin. A
  slow column shows its cells a little later, faintly marked, and never
  holds up the list. Sorting by the column asks for every file of the folder;
  the header shows how far that is, and the list is sorted once all values
  are in.
- **Kept until the file changes.** A value stays with the file until its size
  or its time changes. When the user changes the plugin's settings, its
  values are all asked for again.
- **In the column worker.** The values are computed in an instance of the
  plugin of its own, as a task is ([tasks.md](tasks.md)): a slow column never
  holds up a page. It shares the storage, the settings and the file
  references, not the variables of the pages and the commands. Its own
  variables stay from one request to the next, so it can keep what it learnt
  (a compiled pattern, a cache), until it is let go after five minutes
  without a request, or after a crash.
- **Limits.** A request has a command's limits: 15 seconds without calling
  tessoa, 120 seconds in all. A column cannot ask the user anything: a
  question answers no. It may make requests to its listed websites
  ([network.md](network.md)), which wait for their answer here.
- **When the plugin stops.** The values tessoa has stay; the others show a
  dash, and the header says the extension stopped. The column stays in the
  list, and fills again when the plugin is back.

## Testing

`expect-column` asks the plugin's column for the values of files of the
scenario's folder, as tessoa does for the rows of its list, and checks them;
the files come from `[[files]]`. A file the column's `when` leaves out shows
nothing, and the plugin is not asked for it, as in tessoa. See
[testing.md](testing.md).

```toml
[[files]]
from = "text/three-lines.txt"

[[files]]
from = "text/empty.txt"

[[steps]]
expect-column = { column = "lines", files = ["three-lines.txt", "empty.txt"], values = [3, 0] }
```
