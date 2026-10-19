# Lists, tables and trees

A list, a table or a tree shows rows of data in the style of tessoa's own
file list: the lines of a CSV file, search results, the entries of a JSON
document, a folder outline. The page says how many rows there are and how to
describe row `i`; tessoa asks only for the rows on screen, so a million rows
cost the page what twenty do.

| Node | Shows | Example |
|---|---|---|
| list | rows in one column, optionally with a second, dimmed line | `examples/controls-ts`, `controls-rs` |
| table | rows under a header: columns that sort and that the user resizes | `examples/csv-table-ts` |
| tree | rows with levels: a row with children opens and closes | `examples/json-tree-ts` |

**When to use one.** Beyond two hundred items or so, or as soon as the user
should select several at once, use a list instead of a column of nodes. A
page re-describes every node on every render; a list's rows are asked for
only when they come on screen, and the selection, the scrolling and the keys
are tessoa's.

## Rows

```tsx
<list key="results" count={s.results.length} selection="multi" twoLine
      row={(i) => ({ id: s.results[i].id, cells: [s.results[i].name], detail: s.results[i].place })}
      onSelect={(ids) => { s.selected = ids; }}
      onActivate={(row) => openResult(s, row)} />
```

In Rust the node carries the count, and `Page::rows` describes the rows of
every list of the page by its key:

```rust
fn render(&self) -> Node {
    list(self.results.len()).key("results").selection(Selection::Multi).two_line()
}

fn rows(&self, list: &str, start: usize, count: usize) -> Vec<Row> {
    match list {
        "results" => self
            .results
            .iter()
            .skip(start)
            .take(count)
            .map(|r| Row::new(&r.id, [r.name.as_str()]).detail(&r.place))
            .collect(),
        _ => Vec::new(),
    }
}
```

- **A row is data, not a node.** It has an `id`, its `cells` (one per
  column, in order), and optionally `detail` (the second line, with
  `twoLine`), `depth` and `expander` (in a tree), `actions` (buttons at its
  end) and a `tone` for the whole row. All rows have the same height. A row
  cannot hold buttons, fields or other nodes: to change a row, open a popup
  or put a form beside the list.
- **The id names the row** in every event, in the selection and in
  scenarios. Build it from the item's own id, never from the row's position:
  the selection follows the ids when the rows are sorted or new ones come in.
  At most 128 bytes, unique within the list; a number given as an id in
  TypeScript becomes text.
- **`row` describes, and nothing else.** tessoa calls it (Rust: `rows`) when
  rows come on screen, and a screen above and below, at most 2000 at a time.
  The same data must give the same row. It changes nothing, and asking the
  user or waiting there throws. Keep the data the rows come from in the
  state, sorted the way the rows show, so that row `i` is quick to find.
- **Rows come with the tree.** After every render tessoa gets the rows on
  screen together with the new tree, so a page that removes a row never
  shows the old rows under the new count.
- A list with no rows shows `empty`, such as "Nothing found".

| Cell | TypeScript | Rust |
|---|---|---|
| Text | `"Report.pdf"`, or `{ text, tone, icon }` | `Cell::text("Report.pdf")`, `.tone(Tone::Dim)`, `.icon("note")`; a `&str` or a `String` works as a text cell |
| A number, written as text | `12.5` | `Cell::text(n.to_string())` |
| Nothing | `null` or `undefined` | `Cell::Empty` |
| A file issued to the plugin, drawn as in tessoa's file list: it opens on a double click, can be dragged out and has tessoa's file menu | `{ file: ref }` | `Cell::file(r)` |
| A small picture | `{ image: "images/a.png" }` or `{ image: id }` | `Cell::image("images/a.png")`, `Cell::image(Img::added(id))` |
| A checkbox: a click calls `onRowCheck` | `{ check: true }` | `Cell::check(true)` |
| A progress bar, from 0 to 1 | `{ progress: 0.4 }` | `Cell::progress(0.4)` |

A text cell shows its first 1 KiB. The buttons at the end of a row show
while the pointer is over it, at most four: `actions: [{ id: "open", icon:
"eye", label: "Open" }]` (Rust `.action("open", "eye", "Open")`); a click
calls `onRowAction(row, action)` (Rust `Event::RowAction { list, row, action
}`). A click on a check cell calls `onRowCheck(row, column, checked)` (Rust
`Event::RowCheck`) with the new state; the cell shows what the page's next
rows say, so change the data the cell comes from.

A row action shows its label as a tooltip; an empty label shows no tooltip.
In a two-line table, `detail` is below the first text cell only, not repeated
under the other columns. A table without `twoLine` stays single-line.

When rows of a long list are slow to describe, give `rowId={(i) => ...}`
(Rust: `Page::row_ids`): tessoa asks for ids alone, up to 10,000 at a time,
when the user selects a range of rows that were never on screen, or all of
them. Without it, the ids come from `row`.

## Columns

A table has its columns; a list or a tree given columns uses the first one
only.

```tsx
columns={[
  { id: "name", title: "Name", width: "fill", sortable: true },
  { id: "size", title: "Size", width: 6, align: "end", sortable: true },
  { id: "done", title: "Done" },
]}
```

```rust
table(n, [
    column("name", "Name").width(SizeHint::Fill).sortable(),
    column("size", "Size").width(SizeHint::Em(6.0)).align(Align::End).sortable(),
    column("done", "Done"),
])
```

The id names the column in `onSort` and `onRowCheck`: unique within the
node, not empty. `width` is `"auto"` (the default), `"fill"` or a number of
em, and is only where the column starts: the user drags the line between two
headers to resize it, or double-clicks it to fit the rows on screen, and
tessoa remembers the width for that list and column, also after a restart.
`align` is `start` (the default), `center` or `end`; numbers read best at the
end. In Rust the column is `column(id, title)`, because `col` is the node
that stacks its children.

## Selection

tessoa holds the selection, by row id, and draws it at once, without waiting
for the page. `selection` is `"single"` (the default), `"multi"` or
`"none"`. The gestures are tessoa's file list's:

- A click selects the row alone. Ctrl and a click adds or removes it; Shift
  and a click selects the rows from the last row clicked; Ctrl, Shift and a
  click adds those.
- Up, Down, PageUp, PageDown, Home and End move the focus, and the selection
  follows it; with Shift they select from the last row clicked, with Ctrl
  only the focus moves. Space adds or removes the focused row. Ctrl+A selects
  every row. Enter opens the focused row.
- In a tree, Right opens a closed row and goes to the first child of an open
  one; Left closes an open row and goes to the parent of another.
- With `"single"` there is one row at most: Shift does what the key does
  alone, Ctrl+A does nothing, and Ctrl and a click on the selected row leaves
  none. With `"none"` only the focus moves, and Enter and a double click
  still open rows.

The page hears `onSelect(ids)` (Rust `Event::Select { list, ids }`) whenever
the set changes, with the ids in the order of the rows, and keeps them for
what it does with the selection: a Copy button, a menu, a detail beside the
list. The rows the page takes away stay in the selection until the user
selects again; they come last.

The page can set the selection too: `selected={ids}` (Rust
`.selected_rows(ids)`, at most 10,000) replaces the user's selection when it
differs from what the page's previous render gave. A page that gives back
the selection it heard therefore keeps the user's, and clears it by setting
its copy to `[]`, for instance when it shows another file:

```tsx
selected={s.selected}
onSelect={(ids) => { s.selected = ids; }}
```

Enter and a double click call `onActivate(row)` (Rust `Event::Activate {
list, row }`). A menu on the list (`menu`, as on any node) acts on the
selection: a right click on a row that is not selected selects it alone
first, and tessoa opens the menu once the page has heard the new selection.

## Sorting

tessoa does not sort: it has only seen the rows on screen. A click on the
header of a sortable column calls `onSort(column, descending)` (Rust
`Event::Sort { list, column, descending }`): first ascending, then the other
way round. The page sorts its data and gives `sort={{ column, descending }}`
(Rust `.sort(column, descending)`), which tessoa shows as an arrow in the
header. Keep the order stable, ties in a fixed order, so that the same data
always shows the same rows; the selection follows the ids.

## Trees

A tree is a list whose rows have a `depth` (0 at the top) and an `expander`:
`"none"`, `"collapsed"` (it has children that do not show) or `"expanded"`.
The page decides which rows exist: opening a row means its children become
rows below it. A click on the expander, and Right or Left, call
`onExpand(row, open)` (Rust `Event::Expand { list, row, open }`).

`TreeModel` does that work: it keeps the items, each with an id unique in the
whole tree and a value of the page's choosing, remembers which are open, and
answers by row number with the depth and the expander filled in.

```tsx
const files = new tessoa.pages.TreeModel<string>();
files.setRoots([["docs", "Documents"], ["todo", "To do.txt"]]);
files.setChildren("docs", [["docs/report", "Report.docx"]]);

<tree key="files" count={s.files.count()}
      row={(i) => s.files.row(i, (id, label) => ({ cells: [label] }))}
      onExpand={(row, open) => { s.files.expand(row, open); }} />
```

```rust
let mut files = TreeModel::new();
files.set_roots([("docs", "Documents".to_string()), ("todo", "To do.txt".to_string())]);
files.set_children("docs", [("docs/report", "Report.docx".to_string())]);

// In render:
tree(self.files.count()).key("files")
// In rows:
self.files.rows(start, count, |id, label| Row::new(id, [label.as_str()]))
// In event:
Event::Expand { row, open, .. } => { self.files.expand(&row, open); }
```

| TypeScript | Rust | Does |
|---|---|---|
| `setRoots([[id, value], ...])` | `set_roots([(id, value), ...])` | Replaces the items at the top. An item already in the tree, by id, keeps what is below it and stays open or closed. |
| `setChildren(parent, items)` | `set_children(parent, items)` | Replaces the children of `parent` the same way; false when the tree has no item `parent`. |
| `setLazy(id)` | `set_lazy(id)` | The children of `id` come later: it gets an expander, and opening it makes `expand` return true. False when the tree has no item `id`. |
| `expand(id, open)` | `expand(id, open)` | Opens or closes `id`, as `onExpand` asks; true when it opened and its children are still to come. |
| `isOpen(id)`, `get(id)`, `set(id, value)`, `clear()` | `is_open`, `get`, `get_mut`, `clear` | |
| `count()` | `count()` | How many rows show: the tree node's `count`. |
| `row(i, describe)`, `rows(start, count, describe)`, `rowIds(start, count)` | `row(i)`, `rows(start, count, describe)`, `row_ids(start, count)` | Rows for the node; `describe` makes a row from the item's id and value, and the tree sets the id, the depth and the expander. |

**Children read when they open.** Set the items first, then mark the ones
whose children come later: `setLazy` and `setChildren` name an item that is
already in the tree. For an id that is not there, they return false, leave the
tree unchanged and log a warning naming the method and the missing id.
When the user opens such an item, `expand` returns true and the page sets its
children, at once or, when finding them takes long, with a row such as
"Loading…" first and the real children when a task has found them:

```tsx
/** Put items below `parent`; the ones that have children of their own get an expander. */
function place(tree: tessoa.pages.TreeModel<Entry>, parent: string, items: [string, Entry][]): void {
  tree.setChildren(parent, items);
  for (const [id, entry] of items) {
    if (entry.hasChildren) tree.setLazy(id);
  }
}

// On the tree node:
onExpand={(row, open) => {
  if (s.tree.expand(row, open)) place(s.tree, row, childrenOf(row));
}}
```

`examples/json-tree-ts` reads the entries of an object or an array of a JSON
file this way.

## Keys and the focus

A click on a row puts the focus in the list, and Tab reaches it like any
control. The list then takes the arrows, PageUp, PageDown, Home, End, Space,
Ctrl+A and Enter, and in a tree Left and Right; other keys go to the page when
it declares them, so a page that declares `Ctrl+c` hears it while the list
has the focus. Typing does not jump to a row.

## Limits

| | |
|---|---|
| Rows | at most 1,000,000 |
| Columns | at most 32 |
| A row's id | at most 128 bytes, unique within the list |
| A cell's text | the first 1 KiB shows |
| Buttons at the end of a row | at most 4 |
| Rows tessoa asks for at once | at most 2000 and 1 MiB; ids alone, at most 10,000 |
| A selection the page gives | at most 10,000 ids |

The SDK hands tessoa at most the rows it asked for, and the runner checks
them before tessoa sees them: a row that breaks a limit (a long id, two rows
with one id, a fifth button, a file the plugin was not given) leaves the rows
of that request out, and the plugin log says why. A list that stays
empty although it has a count usually lacks `row`, or in Rust `Page::rows`:
the log then says "described nothing from row 0 of list ...".

## Testing

A scenario sees 20 rows of each list on its screen, as the user does, and
clicks only those: `scroll` brings others up. `click-row` (with `shift` and
`ctrl`), `activate-row`, `check-row`, `row-action`, `sort`, `expand` and the
keys play the user; `expect-rows` and `expect-selected` check. See
[testing.md](testing.md#lists).

`tessoa-plugin render` writes the rows on screen below the node, the cells
apart by `|`, and how many rows are not shown; `--rows 0..3` writes only
those rows:

```
  table key=rows count=25 selection=multi columns=[c0 "name" sortable, c1 "city" sortable, c2 "age" end sortable, c3 "joined" sortable] sort=c2 selected=[] empty="Open a CSV file to see its rows here." focused grow
    r0 "Ada Lovelace" | "London" | "36" | "2021-03-04"
    r1 "Alan Turing" | "Wilmslow" | "41" | "2020-11-12"
    r17 "John von Neumann" | "Budapest" | "53" | "2016-06-18"
    … 22 more rows
```

A tree's rows are indented by their depth, with `[+]` before a row that
opens and `[-]` before an open one (`--rows 3..9`):

```
    tree key=json count=12 selected=["$.author"] empty="Open a JSON file to see it here." focused
      … 3 rows before
      [+] "$.keywords" "keywords: [3 items]"
      [-] "$.author" "author: {2 keys}" selected
        "$.author.name" "name: \"Example\""
        "$.author.url" "url: \"https://example.com/\""
      [-] "$.plugins" "plugins: [2 items]"
        [+] "$.plugins[0]" "0: {3 keys}"
      … 3 more rows
```
