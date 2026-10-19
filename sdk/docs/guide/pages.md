# Pages

A page is a whole tab, or a section of the sidebar, that a plugin fills: a
calculator, a shelf of files, a task board, a table of a CSV file. The plugin
describes what the page shows as a tree of nodes; tessoa draws it with its
own widgets, in the user's theme, component style, language and scale, and
tells the plugin what the user did. The plugin never draws, never sees pixels
or fonts, and needs no permission for the page itself.

Complete pages to copy from, each with page scenarios in its
`tests/scenarios`:

| Example | Shows |
|---|---|
| `examples/calculator-ts`, `examples/calculator-rs` | The same page in both languages: buttons, declared keys, the clipboard, state that survives a restart. The TypeScript one also has its own colours, `[theme]` in its `plugin.toml`. |
| `examples/controls-ts`, `examples/controls-rs` | Every control, the same page in both languages: choices, values, a colour, a date, an image, rich text, a grid, a fold, a popup, a split, a list, a table and a tree. |
| `examples/shelf-ts` | Files: a drop zone, file rows, copy, move and trash done by tessoa, a command that opens the page. |
| `examples/kanban-ts` | A board: draggable cards, a card menu, a field of several lines, a question before deleting, data shared by every page. |
| `examples/csv-table-ts` | A file the user picks, read into a table that sorts; selected rows copied. |
| `examples/json-tree-ts` | A file the user picks, shown as a tree whose branches are read when they open (`TreeModel`). |
| `examples/github-info-ts` | A website: a request from a page's handler, its answer and its errors, a link the user opens. |

## Command or page

| | Command | Page |
|---|---|---|
| The user | selects files and picks the command from the context menu | opens the page from tessoa's menus (a right click on the tab bar, or on an empty spot of a folder), or a command opens it |
| It runs | once, on those files, then ends | as long as the page is open |
| Good for | renaming, counting, converting, tagging | a tool, a place to collect things, a board |

A plugin can have both. The shelf has a command, "Add to the shelf", that
puts the selected files on the shelf and shows it. Work that takes longer
than a second runs as a task beside either ([tasks.md](tasks.md)).

## The rules of a page

1. **Describe the whole page, every time.** `render` returns the full tree;
   tessoa compares it with the previous one and redraws what changed. There
   is no partial update and nothing to compare yourself.
2. **Handlers change the state; the SDK renders.** After every event and
   every command, every open page of the plugin is rendered again. Do not ask
   for a render, and do not update another page by hand: two board pages that
   show the same data stay in step by themselves.
3. **`render` only describes.** It runs after every event for every open
   page, so it changes nothing: no changes to the state, no storage writes,
   no questions. Keep it quick.
4. **Give every node the user acts on a key.** Events name a node only by
   its key: in Rust a button, field, checkbox, control, list, menu or drop
   zone without a key can send nothing. A key is unique within the whole
   page, not only among siblings, so in a list build it from the item's id
   (`card-12`), never from its position. In TypeScript a node with a handler
   and no key gets a key from its position (`#0.2`); that key changes when
   nodes before it come and go, which breaks scenarios and loses what the
   user typed. Do not start your own keys with `#`.
5. **A control shows the value the page gives it.** A checkbox, a switch, a
   choice, a number input, a slider, a colour, a date, a fold and a popup
   show what the render says. When the user changes one, tessoa shows the
   user's value at once and sends the event; once the page has handled it,
   the control shows what the page renders. So keep the new value in the
   state in the handler: a page that does not puts the control back. See
   [Controls](#controls).
6. **A text field keeps what the user typed.** It is the exception to rule 5,
   and so is the bar of a split. `value` is what the page gives the field;
   the field keeps the user's text until a render gives it a value other
   than the render before it gave. `onSubmit` (Enter, or Ctrl+Enter in a
   field of several lines) receives the field's text. To empty the field
   after a submit, keep its text in the state from `onInput` and set it to
   `""`: a page that always gives `value=""` can never empty the field,
   because the value never changes.
7. **A page never waits for the user.** In TypeScript,
   `await tessoa.ui.confirm(...)` in a handler shows the question and pauses
   only that handler: the page and the plugin's other pages keep working,
   and the handler goes on with the answer. The file pickers, a new file and
   `tessoa.net.fetch` are awaited the same way. Data may have changed by
   then; read it again after the `await`. One `await` is different: after
   `await task.done` the handler goes on when the task ends, which is not
   something the user did, so it can change the state and start tasks but
   can no longer ask the user or go to the network
   ([tasks.md](tasks.md#a-page-that-waits-for-a-task)). In Rust,
   `pages::confirm` returns a dialog id at once, and the answer arrives later
   as `Event::Dialog`; the pickers, a new file and a request have `_start`
   functions whose answers arrive as events too. Ask only before something
   that cannot be undone; a Cancel button on the page is better than a
   question.
8. **Declare the keys the page handles.** While the page has the focus and no
   control takes the key, the declared keys go to the page; every other key
   works as it does elsewhere in tessoa. See [Keys](#keys).
9. **Files come from the user.** A page shows a file only by a file reference
   the plugin got from the user: files dropped on a drop zone, files a
   command kept with `files.keep`, files picked in tessoa's file picker,
   files the plugin created in a folder the user picked. References stay
   valid across restarts. The plugin never sees a path. See [Files](#files)
   and [files.md](files.md).
10. **Long lists are rows, not nodes.** Beyond two hundred items or so, show
    them in a list, a table or a tree: tessoa asks for the rows on screen
    only, so a million rows cost what twenty do. See [Lists](#lists).
11. **Every page of the plugin shares one instance.** A variable outside the
    page state (a module-level variable in TypeScript, a `static` or
    `thread_local!` in Rust) is shared by all pages of the plugin, and lost
    when the plugin restarts or stops. Keep what must last in the storage.
    Tasks and columns run in other instances and see none of these
    variables.
12. **Save what the page needs after a restart.** What `save` returns is
    kept by tessoa and given back to `open` when tessoa starts again; `save`
    runs with every render and keeps at most 16 KiB. Data that every page
    shows, such as the cards of a board, belongs in the plugin's storage
    (`tessoa.storage`, Rust `storage`; no permission needed); the page state
    is for what belongs to one tab, such as a draft. The storage holds at most
    1 MiB, 256 KiB per value, with keys of up to 128 bytes; a write beyond
    that fails (TypeScript throws a `StorageError`, Rust returns an `Err`).
13. **tessoa chooses the look.** A page names sizes (`caption`, `body`,
    `title`, `display`), tones (`dim`, `accent`, `danger`, `success`),
    spacing (`xs` to `l`) and button kinds; it cannot give a node its own
    colour, font or pixel size, and there is no `style` or `className`. The
    plugin may give all its pages a palette in `plugin.toml`; see
    [Colours](#colours). `tessoa.ui.language()` (Rust `ui::language()`)
    returns the language code for the plugin's own texts.

A render that fails, or a tree that breaks the rules below, shows an error in
place of the page. An event that fails goes to the plugin log; the page stays.
A plugin that crashes or hits a limit stops every page it has open, and the
user can reload them.

## The tree

Containers and the basic nodes:

| Node | JSX | Rust | Notes |
|---|---|---|---|
| column | `<column>` | `col([...])` | Children top to bottom. |
| row | `<row>` | `row([...])` | Children left to right. |
| card | `<card>` | `card([...])` | A column with a frame and a background. |
| divider | `<divider />` | `divider()` | A thin line across the parent. |
| spacer | `<spacer />` | `spacer()` | Empty space; with `grow`, pushes its siblings apart. |
| text | `<text>Hi</text>` | `text("Hi")` | `size` (default `body`), `tone` (default `normal`), `align` (`start`, `center`, `end`), `wrap` to break lines instead of cutting the text off. |
| button | `<button onClick={f}>Add</button>` | `btn("Add")` | `kind` (`normal`, `primary`, `danger`, `quiet`), `icon`, `disabled`. |
| icon-button | `<icon-button icon="x" label="Close" />` | `icon_btn("x", "Close")` | Shows only the icon; the label is the tooltip. |
| text-input | `<text-input value={v} />` | `text_input(v)` | `placeholder`, `disabled`, `multiline`. |
| checkbox | `<checkbox checked={c}>Done</checkbox>` | `checkbox(c, "Done")` | Shows `checked` as the page gives it: update it from `onToggle`. |
| file-row | `<file-row file={ref} />` | `file_row(r)` | One file, drawn by tessoa: icon, name, size, dimmed when gone. Double click opens it; the user can drag it out; its context menu is tessoa's file menu. `selected`, `draggable` (default true). |
| drop-zone | `<drop-zone accepts="files">` | `drop_zone(Accepts::Files, [...])` | Takes `files`, `items` or `both`; `hint` shows while nothing is dragged over it. |
| draggable | `<draggable key="c1" payload="c1">` | `draggable("c1", [...]).key("c1")` | Its children can be dragged onto a drop zone that takes items. Give it a key; see [Dragging items](#dragging-items). |

Controls with a value, all shown as the page gives them ([Controls](#controls)):

| Node | JSX | Rust | Notes |
|---|---|---|---|
| dropdown | `<dropdown value={v} options={opts} />` | `dropdown(v, opts)` | A button showing the chosen option; a click opens the options. |
| radio-group | `<radio-group value={v} options={opts} />` | `radio(v, opts)` | The options one below the other, each with a round mark. |
| segmented | `<segmented value={v} options={opts} />` | `segmented(v, opts)` | The options side by side as joined buttons. |
| tabs | `<tabs value={v} options={opts} />` | `tabs(v, opts)` | Only the row of tabs: below it, the page shows what belongs to the chosen one. |
| switch | `<switch checked={on}>Dark</switch>` | `switch(on, "Dark")` | An on/off switch; works like a checkbox. |
| number-input | `<number-input value={n} min={0} max={100} />` | `number_input(n).range(0.0, 100.0)` | `step` (default 1), `decimals` (default: as many as the step has), `suffix` such as `"px"`. Without `min` and `max`: −1,000,000,000 to 1,000,000,000. |
| slider | `<slider value={n} />` | `slider(n)` | `min` (default 0), `max` (default 100), `step` (default 1), `vertical`. |
| color-picker | `<color-picker value="#ff8800" />` | `color_picker(0xff8800)` | A swatch with the colour's code; a click opens tessoa's colour picker. |
| date-picker | `<date-picker value="2026-10-15" />` | `date_picker(Some(Date::new(2026, 10, 15)))` | A button showing the date; a click opens a calendar. `min`, `max`; no value shows an empty button. |

Showing things:

| Node | JSX | Rust | Notes |
|---|---|---|---|
| progress | `<progress value={0.4} />` | `progress(Some(0.4))` | From 0 to 1; without a value, a bar that keeps moving. |
| image | `<image src="images/logo.png" />` | `image("images/logo.png")` | A file of the plugin, or an image it made; `fit`, `alt`. See [Images](#images). |
| rich-text | `<rich-text>Read <b>this</b></rich-text>` | `rich([span("Read "), span("this").bold()])` | Bold, italic, code, colours and links. See [Rich text](#rich-text). |

Containers that do more than arrange:

| Node | JSX | Rust | Notes |
|---|---|---|---|
| grid | `<grid columns={3}>` | `grid(3, [...])` | Children in a grid, filled row by row. A column is as wide as its widest child; room to spare is shared equally. 1 to 32 columns. |
| fold | `<fold title="More" open={o}>` | `fold("More", o, [...])` | A title that opens and closes the children below it. Closed, they stay in the tree but are not drawn and cannot be reached. |
| split | `<split ratio={0.3}>{a}{b}</split>` | `split(0.3, a, b)` | Exactly two children with a bar between them the user can drag; `vertical` puts one above the other. |
| popup | `<popup open={o} title="Rename">` | `popup(o, [...]).title("Rename")` | Its children in a panel over the page. See [Popups](#popups). |

Rows of data ([Lists](#lists)):

| Node | JSX | Rust | Notes |
|---|---|---|---|
| list | `<list count={n} row={(i) => ...} />` | `list(n)` and `Page::rows` | Rows in one column. |
| table | `<table count={n} columns={cols} row={...} />` | `table(n, cols)` and `Page::rows` | Rows under a header whose columns sort and resize. |
| tree | `<tree count={n} row={...} />` | `tree(n)` and `Page::rows` | Rows with levels that open and close. |

Every node takes `key`, `grow`, `w`, `h`, `pad`, `gap`, `justify`, `scroll`,
`menu` and `tooltip` (a line shown when the pointer rests on the node, at
most 1 KiB; an icon button without one shows its label, an image its `alt`);
a container also takes `align` (across its direction; default `stretch`) and
`wrap` (children that do not fit continue on the next line). Only column,
row, card, grid, fold, popup, drop-zone and draggable have children, and a
split has its two; the content of `<text>`, `<button>`, `<checkbox>` and
`<switch>` is text only, and that of `<rich-text>` text and its spans. In
Rust the attributes are methods:
`text("0").key("display").size(TextSize::Display).align(Align::End)`; a
method that does not apply to the node, such as `.size()` on a button, is
ignored and logged once. Only the attributes in the SDK's declarations exist.

Layout works like a flexible box: children stretch across the direction of
their parent unless `align` says otherwise. The children that `grow` share
the length that is left along it in equal parts, whatever their content, as
with CSS `flex: 1 1 0`; none is made shorter than its content needs. Three
buttons that grow in a row are equally wide, whether their labels are
"0" or "Copy". `w` and `h` are `"auto"` (as large as the content), `"fill"`
(as large as the parent allows) or a number of em. Give a list, a table, a
tree or a split a height (`h`, or `grow` in a column that has one): their
content scrolls inside them.

A small menu: any node can have `menu={[{ id: "rename", label: "Rename" }, { id:
"delete", label: "Delete", danger: true }]}` (Rust: `.menu([item("rename",
"Rename"), item("delete", "Delete").danger()])`). A right click on the node
opens it. A left click on a button or an icon button that has a menu opens
the menu too, and then sends no click, so give a "more" button the menu and
no `onClick`. Choosing an item calls `onMenu(id)` (Rust: `Event::Menu { key,
item }`). Give the node a key, and the items ids that are unique within the
menu. A list's menu acts on its selected rows.

Icons: `icon` on a button or an icon button, and `icon` of a page in
`plugin.toml`, take a name from tessoa's icon set, drawn in the user's theme.
The names: `plus`, `close`, `arrow-left`, `arrow-right`, `arrow-up`,
`arrow-down`, `chevron-down`, `chevron-right`, `check`, `search`, `menu`,
`more`, `reload`, `undo`, `copy`, `cut`, `paste`, `new-folder`, `rename`,
`delete`, `play`, `note`, `sliders`, `eye`, `sun`, `gear`, `image`,
`keyboard`, `monitor`, `info`, `pin`, `question`, `tag`, `filter`, `puzzle`.
Later versions only add names. A name tessoa does not know shows the puzzle
piece: in TypeScript it does not build (the attribute's type is
`tessoa.pages.IconName`), in `plugin.toml` `tessoa-plugin check` reports it,
in Rust nothing catches it, so copy the name from this list. The options of a
choice, the cells of a row and the buttons at the end of a row take the same
names.

Limits: 4096 nodes, 64 levels below the root (the root is level 0, its
children level 1) and 1 MiB per tree; a tree beyond any of them fails the
page. At most 20 renders a second per page, more are merged. A TypeScript
page handles every node in every render: a few hundred nodes take a few
milliseconds, thousands are slow. Rows of a list do not count as nodes.

## Controls

A control shows the value the page gives it; the user's change arrives as an
event, and the page keeps the new value in its state (rule 5). The value
arrives already as tessoa holds it: a number on the control's step and within
its range, a colour as six hex digits, a date that the calendar offers.

| Control | Its value | TypeScript | Rust |
|---|---|---|---|
| checkbox, switch | `checked` | `onToggle(checked)` | `Event::Toggle { key, checked }` |
| dropdown, radio-group, segmented, tabs | `value`: the id of the chosen option, `""` for none | `onChange(option)` | `Event::Choose { key, option }` |
| number-input | `value` | `onChange(value)`: on Enter, when the field loses the focus, and on the arrows; not while the user types | `Event::Value { key, value }` |
| slider | `value` | `onChange(value)`: while it is dragged and once more when it is let go | `Event::Value { key, value }` |
| color-picker | `value`: `"#rrggbb"` (also `"#rgb"` or a number) in TypeScript, `0xrrggbb` in Rust | `onChange("#rrggbb")`, in lower case | `Event::Color { key, color }` |
| date-picker | `value`: `"2026-10-15"` or a `Date` in TypeScript, `Option<Date>` in Rust | `onChange("2026-10-15")`, or `undefined` when cleared | `Event::Date { key, date }` |
| fold | `open` | `onToggle(open)` | `Event::Toggle { key, checked }` |
| split | `ratio`, the share of the first child | `onChange(ratio)` when the bar is let go | `Event::Split { key, ratio }` |
| popup | `open` | `onDismiss()` | `Event::Dismiss { key }` |

```tsx
<dropdown
  key="format"
  value={s.format}
  options={[{ id: "iso", label: "2026-10-15" }, { id: "us", label: "10/15/2026" }]}
  onChange={(option) => {
    s.format = option;
  }}
/>
```

```rust
// In render:
dropdown(&self.format, [opt("iso", "2026-10-15"), opt("us", "10/15/2026")]).key("format")
// In event:
Event::Choose { key, option } if key == "format" => self.format = option,
```

The options of a choice: at most 256, each with an `id` unique within the
node and not empty, a `label`, optionally an `icon` and `disabled`. In
TypeScript a plain string is an option whose id is its label; in Rust
`opt(id, label).icon("tag").disabled(true)`. The user cannot choose a
disabled option, nor the one already chosen.

While a slider or the colour picker is dragged, only the newest value waits
to be delivered: a slow page is not flooded, and it hears the value the drag
ended on. A split's bar is told once, when it is let go; tessoa keeps where
the user put it, and remembers it after a restart, until the page gives
another ratio, as a text field keeps its text. Give a split a key.

`disabled` greys out a choice, a number input, a slider, a colour or a date
picker; it takes no clicks and no keys.

## Rich text

Text with bold, italic and code spans, colours and links, wrapped like text
with `wrap` and aligned with `align`:

```tsx
<rich-text key="about" wrap onLink={(action) => showMore(action)}>
  Read <b>this</b> on <a href="https://example.com/">the site</a> or see <a action="more">more</a>.
</rich-text>
```

```rust
rich([
    span("Read "),
    span("this").bold(),
    span(" on "),
    url("the site", "https://example.com/"),
    span(" or see "),
    action("more", "more"),
    span("."),
])
.wrap()
.key("about")
```

In TypeScript the spans are the children: text and `<b>`, `<i>`, `<code>`,
`<span tone="accent">` and `<a>` around text, which may nest; neighbouring
text in the same style becomes one span. These five elements exist only
inside `<rich-text>`. JSX drops the line breaks and the spaces at the start of
a line between elements, so write a space that must stay as `{" "}` or keep
the sentence on one line. In Rust each span is made with `span(text)` and
`.bold()`, `.italic()`, `.code()`, `.tone(..)`.

A link leads to a web address (`<a href>`, Rust `url(text, address)`) or to
the page (`<a action>`, Rust `action(text, id)`). A click on a web address
goes to tessoa, which shows the user the whole address and opens it in the
browser only when they agree; the page hears nothing, and the address need
not be declared anywhere. Only `http` and `https` addresses are links. A
click on an action calls `onLink(action)` (Rust: `Event::Link { key, action }`);
it is a button that looks like a link. Every link is a stop for the Tab key,
and Enter follows it. A handler may also offer an address itself with
`tessoa.ui.openUrl(url)` (Rust `ui::open_url`), while it handles something the
user did: a click, a key, a menu choice.

At most 2048 spans and 64 KiB of text in one rich text.

## Images

An image node shows a picture file of the plugin or an image the plugin
made:

```tsx
<image src="images/logo.png" w={4} h={4} alt="The logo" />
<image src={tessoa.images.add(svgText)} fit="cover" />
```

- A file: PNG, JPEG, GIF (its first frame), WebP, BMP or SVG, written with
  `/` from the plugin's directory. Keep the pictures in the project's `images`
  folder: `tessoa-plugin package` takes that folder whole, and a file
  elsewhere stays out of the package.
- An image the plugin made: `tessoa.images.add(bytes)` with the bytes of one
  of those formats, or the text of an SVG; `tessoa.images.addRgba(width,
  height, pixels)` with 4 bytes per pixel. Rust: `images::add`,
  `images::add_rgba`, then `image(Img::added(id))`. The number is valid until
  the plugin stops: a plugin that starts again adds its images again, so do
  not save the numbers. `remove(id)` frees one; a node that still names it
  shows an empty frame.
- At most 8 MiB and 4096 by 4096 pixels per image, and 64 MiB for all the
  images a plugin made, counted decoded; `add` beyond that fails with
  `quota`.
- `fit` is CSS `object-fit`: `contain` (the default), `cover`, `fill` or
  `none`. Without `w` and `h` the image takes its own size in pixels,
  scaled as the rest of tessoa is; an SVG's size is its `width` and
  `height`, else its `viewBox`.
- `alt` says what the image shows; it is the tooltip when the node has none.

A cell of a row can show a small image too (`{ image: src }`, Rust
`Cell::image(src)`); see [lists.md](lists.md).

## Popups

A popup shows its children in a panel over the page while `open` is true:

```tsx
<button key="rename" onClick={() => { s.renaming = true; }}>Rename…</button>
<popup key="rename-popup" open={s.renaming} title="Rename" anchor="rename" w={20}
       onDismiss={() => { s.renaming = false; }}>
  <text-input key="name" value={s.draft} onInput={(t) => { s.draft = t; }} />
  <button key="save" kind="primary" onClick={() => save(s)}>Save</button>
</popup>
```

- It covers the page, and only the page: a faint layer lies over the rest of
  the page, which cannot be clicked while the popup is open, and the tab bar,
  the sidebar and the other tabs work as before.
- Escape, a click outside the popup and its close button (shown with a
  `title`) call `onDismiss` (Rust `Event::Dismiss`). The popup stays open
  until the page renders `open` false, so set it in the handler.
- `anchor` is the key of a node to show the popup below; without it the
  popup is centred on the page. `w` is the width of the panel.
- While it is open, Tab and Shift+Tab go round the controls inside it only.
- One popup of a page is open at a time. Open it from a button the user
  clicks, not when the page opens: a page that covers itself at once shows
  nothing else.

## Lists

A list, a table or a tree shows rows of data: the files of a folder, the
lines of a CSV file, the entries of a JSON document. The page says how many
rows there are and how to describe row `i`; tessoa asks only for the rows on
screen, and draws them in the style of its own file list.

```tsx
<table key="people" count={s.people.length} sort={s.sort}
       columns={[{ id: "name", title: "Name", width: "fill", sortable: true },
                 { id: "age", title: "Age", align: "end", sortable: true }]}
       row={(i) => ({ id: s.people[i].id, cells: [s.people[i].name, s.people[i].age] })}
       selection="multi" onSelect={(ids) => { s.selected = ids; }}
       onSort={(column, descending) => sortPeople(s, column, descending)} />
```

In Rust the node says how many rows there are, and `Page::rows` describes
them:

```rust
fn render(&self) -> Node {
    table(self.people.len(), [column("name", "Name").width(SizeHint::Fill).sortable()])
        .key("people")
        .selection(Selection::Multi)
}

fn rows(&self, _list: &str, start: usize, count: usize) -> Vec<Row> {
    self.people.iter().skip(start).take(count).map(|p| Row::new(&p.id, [p.name.as_str()])).collect()
}
```

- **Rows are data, not nodes.** A row has an id, one cell per column (text,
  a number, a file, a small image, a checkbox or a progress bar), and
  optionally a second line, a level and an expander (in a tree), up to four
  buttons at its end, and a tone. Rows cannot hold other nodes.
- **The id names the row**: in the events, in the selection, in scenarios.
  Make it from the item's own id, never from its position, so that the
  selection stays with the rows when they are sorted.
- **tessoa holds the selection** and draws it at once: clicks, Ctrl, Shift,
  the arrows, Space and Ctrl+A work as in tessoa's file list. The page hears
  the selected ids in `onSelect` (Rust `Event::Select`), and may give a
  selection of its own with `selected`.
- **The page sorts and opens.** A click on a sortable column's header calls
  `onSort(column, descending)` (Rust `Event::Sort`); the page sorts its data
  and renders `sort` to match. A tree's expander calls `onExpand(row, open)`
  (Rust `Event::Expand`); the page changes which rows exist. A `TreeModel`
  does that work for a tree.
- Enter and a double click call `onActivate(row)` (Rust `Event::Activate`).

[lists.md](lists.md) has the whole of it: the cells, selection, sorting,
column widths, trees and `TreeModel`, the limits.

## Dragging items

To let the user reorder items or move them between lists, put each item in a
`draggable` inside a `drop-zone` that takes `items`, and give every draggable
the item's id as its key and as its payload.

```tsx
<drop-zone key={`list-${list.id}`} accepts="items" onDropItem={(id, before) => move(id, list.id, before)}>
  {list.cards.map((c) => (
    <draggable key={c.id} payload={c.id}>
      <text>{c.title}</text>
    </draggable>
  ))}
</drop-zone>
```

`onDropItem(payload, before)` (Rust: `Event::DropItem { zone, payload,
before }`) receives the payload of the item that was dropped and the key of
the draggable it was dropped in front of, `undefined` (Rust `None`) at the
end. Only the zone's own children that are draggable and have a key count,
and never the one being dragged: a heading at the top of the zone is not
named, and a draggable without a key is never named, so without keys every
item lands at the end. Take the item out of its old place first, then insert
it in front of `before`, as `examples/kanban-ts` does. Rows of a list cannot
be dragged between lists.

## Keys

A key is written the way it types: a character such as `8`, `+` or `a` (`A`
with Shift), or one of the names `Enter`, `Escape`, `Backspace`, `Delete`,
`Tab`, `Space`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `Home`,
`End`, `PageUp`, `PageDown` and `F1` to `F12`. `8` covers the 8 of the main
keyboard and of the numeric keypad. `Ctrl+`, `Alt+` and `Meta+` in front add
a modifier (`Ctrl+z`); `Shift+` does so only in front of a name
(`Shift+Tab`). Declare `a` and `A` both when the case should not matter. A
pressed key arrives as it was declared: `k.key === "Ctrl+c"`. Write both
`Ctrl+c` and `Meta+c` for a shortcut that should work on every platform.
Ctrl+Shift combinations and Ctrl+Tab always stay with tessoa. The keys may
depend on the state: in TypeScript `keys` can be a function of it.

```ts
keys: ["0", "1", "+", "Enter", "Backspace", "Escape", "Ctrl+c", "Meta+c"],
onKey: (c, k) => press(c, k.key),
```

**The focus.** Tab and Shift+Tab move the keyboard focus through the
controls that have a key, in the order of the tree, round from the end to
the start: buttons, icon buttons, text fields, checkboxes, switches, the four
choices, number inputs, sliders, colour and date pickers, lists, tables and
trees, unless they are disabled; the title of a fold; every link of rich
text. The children of a closed fold or popup are not on the way; while a
popup is open, only its own controls are. A click puts the focus in a text
field, a number input, a slider or a list; a click on a button, a checkbox, a
switch or a choice does not, so a page that clicks "7" and then hears Enter
still gets Enter. tessoa draws the focus frame only when the keyboard moved
the focus.

The control with the focus takes its own keys first: Enter and Space press a
button, Space turns a checkbox or a switch over, Enter and Space open or close
a fold, Enter follows a link, Up and Down move a radio group and Left and
Right segmented buttons and tabs to the next option, Enter, Space and
Alt+ArrowDown open a dropdown, a colour or a date picker, Up and Down step a number input
(PageUp and PageDown ten steps), the arrows step a slider (PageUp and
PageDown a tenth of its range, Home and End to its ends), and a list takes
the arrows, PageUp, PageDown, Home, End, Space, Ctrl+A and Enter. A text field
with the focus takes every key except Tab. Every other key goes to the page
when it declares it; Tab goes to the page only when it declares Tab and no
control has the focus.

## Where a page opens

`where` in the page's `[[pages]]` entry says where the page goes when the user
opens it from tessoa's menus or from the settings, and when the plugin opens
it with `pagesHost.open`: `"tab"`, a new tab in the focused pane (the
default), or `"sidebar"`, a new section at the bottom of the sidebar. The
first place in the list decides. Pick `sidebar` for something the user keeps
beside the files, such as a shelf or a small tool; pick `tab` for a page that
needs room, such as a board. Either way the user can drag an open page from
the tabs to the sidebar and back. A `single` page that is already open in the
window comes to the front instead, wherever it is.

```toml
[[pages]]
id = "shelf"
title = "Shelf"
single = true
where = ["sidebar"]
```

## Colours

A page shows tessoa's colours unless the plugin gives its pages a theme in
`plugin.toml`: one palette for tessoa's light themes, one for its dark
themes, or only one, which is then used with every theme.

```toml
[theme.light]
background = "#ffffff"
text = "#1f1d2b"
accent = "#5b3fd6"

[theme.dark]
background = "#1e1b2e"
text = "#ece8ff"
accent = "#b18cff"
surface = "#2a2640"
```

| Key | Colours | |
|---|---|---|
| `background` | behind the whole page | required |
| `text` | text | required |
| `accent` | primary buttons, accent text (the `accent` and `success` tones), the selection, the focus frame | required |
| `surface` | cards, text fields, small menus | optional |
| `border` | frames and dividers | optional |
| `text-dim` | dim text (the `dim` tone), placeholders | optional |
| `danger` | danger buttons, danger menu items, danger text (the `danger` tone) | optional |

Colours are written `"#rrggbb"`. tessoa derives an optional colour that is
left out from the three required ones. A theme with a required key missing
or a colour written any other way is not used at all, and the pages show
tessoa's colours; a key that is not in the table is ignored. Both are
reported by `tessoa-plugin check`, by every command that builds, and in the
plugin log.

The theme colours the content of the pages only, the small menus, the colour
picker, the calendar and the popups of the pages included. The tab bar and
the sidebar around a page stay tessoa's, and so do the tooltips, tessoa's
dialogs, the shapes, corners and shadows of the component style the user
chose. A component style may also draw some parts in its own way, so not
every colour of the theme is used with every style. The plugin's colours are
the default; the user can switch a plugin's pages to tessoa's colours in
Settings › System › Extensions.

Keep the colours readable: text needs a contrast of at least 4.5:1 against
the background and against the surface, and the accent at least 3:1 against
the background, so that accent text, the selection and the focus frame show.
`[theme.light]` needs a background lighter than its text, `[theme.dark]` one
darker. `tessoa-plugin check` reports a palette that misses any of these.

## Files

| What | TypeScript | Rust | Permission |
|---|---|---|---|
| Files dropped on a zone | `onDropFiles={(files) => ...}` on a `drop-zone` | `Event::DropFiles { zone, files }` | none |
| Files the user picks | `await tessoa.files.pickFiles({ title, extensions, multiple })` | `files::pick_files_start`, then `Event::PickedFiles` | none |
| Keep a selected file in a command | `tessoa.files.keep(handle)` | `files::keep(handle)` | `files.keep` |
| Name, size, date, whether it still exists | `tessoa.refs.info(refs)` | `refs::info(&refs)` | none |
| Read a referenced file | `tessoa.refs.read(ref)`, `readAll(ref)` | `refs::read`, `refs::read_all` | `files.read-selected` |
| Create a file in a folder the user picks | `await tessoa.files.pickFolder(...)`, then `await tessoa.files.createFile(folder, path, data)` | `files::pick_folder_start`, `files::create_file_start` | `files.create` |
| Copy or move to a folder the user picks | `tessoa.refs.copyToPicked(refs)`, `moveToPicked(refs)` | `refs::copy_to_picked(&refs)`, `move_to_picked` | `files.transfer` |
| Move to the recycle bin, after tessoa's confirmation | `tessoa.refs.trash(refs)` | `refs::trash(&refs)` | `files.delete` |
| Open, or show in a new tab | `tessoa.refs.open(ref)`, `reveal(ref)` | `refs::open(r)`, `reveal(r)` | `files.open` |
| Forget references | `tessoa.refs.release(refs)` | `refs::release(&refs)` | none |

A reference is a number. Keep the numbers in the storage, or in what `save`
returns, to remember files across restarts, and release the ones the page
forgets. A file row or a file cell shows only a reference issued to the
plugin; any other number fails the page. Picking and creating files are
explained in [files.md](files.md).

Copy, move and trash return a transfer id at once. tessoa shows its folder
picker or its confirmation, does the work with its progress, conflict
handling and undo, and reports one outcome per file to the page that started
it: `onTransferDone(state, id, outcomes)` in TypeScript, `Event::TransferDone
{ id, outcomes }` in Rust. An outcome is done, skipped with a reason (the
user cancelled, for instance) or failed with a reason: `{ done: true }`,
`{ skipped: why }`, `{ failed: why }` in TypeScript, `TransferOutcome::Done`,
`Skipped(why)`, `Failed(why)` in Rust. A transfer started by a command
reports to no page.

The page itself, the storage, `pagesHost`, images, settings, tasks and the
pickers need no permission; the clipboard needs `clipboard.write`. See
[permissions.md](permissions.md).

## In TypeScript

Pages are written in JSX: name the file `src/plugin.tsx` (a project from the
template builds `src/plugin.ts` or `src/plugin.tsx`, whichever exists). The
state is a plain object that `open` returns and the handlers change in place.

```tsx
interface Counter {
  count: number;
}

tessoa.pages.define<Counter>("counter", {
  open: ({ state }) => ({ count: typeof state === "number" ? state : 0 }),
  save: (c) => c.count,
  keys: ["+"],
  onKey: (c) => {
    c.count += 1;
  },
  render: (c) => (
    <column pad="m" gap="s">
      <text key="count" size="title">{c.count}</text>
      <button
        key="add"
        onClick={() => {
          c.count += 1;
        }}
      >
        Add
      </button>
    </column>
  ),
});
```

- Change the state in place. What a handler returns is ignored (a returned
  promise is waited for), so `return { ...c, count: c.count + 1 }` does
  nothing. There are no hooks: no `useState`, no `useEffect`, nothing to
  import from React.
- `onKey`, `onTransferDone`, `onActivated` and `close` receive the state as
  their first argument. The handlers on elements (`onClick`, `onInput`,
  `onSubmit`, `onToggle`, `onChange`, `onLink`, `onDismiss`, `onMenu`,
  `onDropFiles`, `onDropItem`, and the lists' `onSelect`, `onActivate`,
  `onSort`, `onExpand`, `onRowCheck`, `onRowAction`) receive no state; they
  use the one `render` received, as in the sample. A value control's handler
  is always `onChange`, an on/off one's `onToggle`.
- `render` returns one element, usually a `<column>`. A function that
  returns elements can be used as a component: `<CardView card={c} />`.
- `save` returns any value JSON can hold, and `open` receives it parsed
  (`undefined` for a new page; check its shape, an older version of the
  plugin may have saved it). A `Set` or a `Map` does not survive JSON; save
  an array.
- Handlers may be `async`: `await tessoa.ui.confirm(...)`, the pickers, a new
  file and `tessoa.net.fetch` wait for their answer without holding up the
  page. Calling them in `render`, or in a list's `row`, throws.
- A task's handle (`tessoa.tasks.start`) follows the task: read its `state`
  and `progress` in `render`, which runs again whenever the task moves on.
  See [tasks.md](tasks.md).
- `tessoa.pagesHost.open(kind, args)` opens a page from a command or another
  page, and `open` receives `args` parsed; when the kind is `single` and a
  page of it is open in the window, that page comes to the front instead and
  the arguments reach no page. A `title` function, or
  `tessoa.pagesHost.setTitle(page, title)` when there is none, changes the
  tab's title.
- Commands work as before: export `run(commandId)` next to the pages.
- A script that is not built uses `tessoa.pages.h(type, props, ...children)`
  instead of JSX.

## In Rust

Turn on the SDK's `pages` feature in `Cargo.toml`:

```toml
tessoa-plugin-sdk = { path = "sdk/rust", features = ["pages"] }
```

Without it, `use tessoa_plugin_sdk::pages::*` fails with an unresolved import
and `export_pages!` says what to add. Then implement `Page` and export it
instead of calling `export!`:

```rust
use tessoa_plugin_sdk::pages::*;

struct Counter {
    count: u32,
}

impl Page for Counter {
    fn open(_page: PageId, state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
        let count = state
            .and_then(|s| String::from_utf8(s).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        Counter { count }
    }

    fn event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Click { key } if key == "add" => self.count += 1,
            Event::Key(_) => self.count += 1,
            _ => {}
        }
        Ok(())
    }

    fn render(&self) -> Node {
        col([
            text(self.count.to_string()).key("count").size(TextSize::Title),
            btn("Add").key("add"),
        ])
        .pad(Space::M)
        .gap(Space::S)
    }

    fn keys(&self) -> Vec<String> {
        vec!["+".into()]
    }

    fn save(&self) -> Option<Vec<u8>> {
        Some(self.count.to_string().into_bytes())
    }
}

tessoa_plugin_sdk::export_pages!(Counter => "counter");
```

- One value of the type is one open page. Keep the `page` that `open`
  receives if the page asks questions or sets its title.
- Every event of the page arrives in `event`, named by the node's key; match
  the ones the page cares about and leave the rest to `_ => {}`, which the
  compiler asks for: later versions add events.
- State and arguments are bytes; the format is yours.
- `pages::confirm(page, title, body)` returns a dialog id at once; the answer
  arrives as `Event::Dialog { id, ok }`. The blocking `ui::confirm` is
  refused in a page and reads as cancel; so are the pickers, `create_file`
  and `net::fetch`, whose `_start` twins return an id at once and answer
  with `Event::PickedFiles`, `PickedFolder`, `Created` and `Fetched`.
- A page with a list, a table or a tree implements `Page::rows`; see
  [lists.md](lists.md). A table's columns are `column(id, title)` (`col` is
  the node), and a list's selection is `.selected_rows(ids)` (`.selected` is
  a file row's).
- Every open page also hears what concerns the whole plugin:
  `Event::TaskProgress`, `Event::TaskDone` and `Event::SettingsChanged`,
  after `Plugin::on_event`. A page knows its own tasks by the id
  `tasks::start` returned.
- A plugin with pages and commands: `export_pages!(Shelf => "shelf";
  commands: MyCommands)`, where `MyCommands` implements `Plugin`; tasks and
  columns are listed there too: `export_pages!(Shelf => "shelf"; tasks:
  [("hash", Hash)])`.

## The two languages side by side

| | TypeScript | Rust |
|---|---|---|
| Define a page | `tessoa.pages.define("kind", { ... })` | `impl Page for T` and `export_pages!(T => "kind")` |
| Build the state | `open({ page, state, args })` | `fn open(page, state, args) -> Self` |
| Describe the page | `render(state)` returning JSX | `fn render(&self) -> Node` |
| A button press | `onClick` on the element | `Event::Click { key }` |
| A text field | `onInput(text)`, `onSubmit(text)` | `Event::Input { key, text }`, `Event::Submit { key, text }` |
| A checkbox, a switch, a fold | `onToggle(checked)` | `Event::Toggle { key, checked }` |
| A choice | `onChange(option)` | `Event::Choose { key, option }` |
| A number input, a slider | `onChange(value)` | `Event::Value { key, value }` |
| A colour, a date | `onChange("#rrggbb")`, `onChange("2026-10-15")` | `Event::Color { key, color }`, `Event::Date { key, date }` |
| A split's bar | `onChange(ratio)` | `Event::Split { key, ratio }` |
| A link of rich text | `onLink(action)` | `Event::Link { key, action }` |
| A popup closed by the user | `onDismiss()` | `Event::Dismiss { key }` |
| Rows | `count` and `row={(i) => ...}` | `list(n)`, `table(n, cols)`, `tree(n)` and `fn rows` |
| Rows selected, opened | `onSelect(ids)`, `onActivate(row)` | `Event::Select { list, ids }`, `Event::Activate { list, row }` |
| A header, an expander | `onSort(column, descending)`, `onExpand(row, open)` | `Event::Sort { .. }`, `Event::Expand { .. }` |
| A cell's checkbox, a row's button | `onRowCheck(row, column, checked)`, `onRowAction(row, action)` | `Event::RowCheck { .. }`, `Event::RowAction { .. }` |
| A declared key | `keys` and `onKey(state, key)` | `fn keys(&self)` and `Event::Key(k)` |
| A drop | `onDropFiles(files)`, `onDropItem(payload, before)` | `Event::DropFiles`, `Event::DropItem` |
| A menu item | `onMenu(item)` | `Event::Menu { key, item }` |
| A question | `await tessoa.ui.confirm(...)` | `pages::confirm(...)`, then `Event::Dialog` |
| Picked files, a new file, a request | `await` the call | `_start`, then `Event::PickedFiles`, `PickedFolder`, `Created`, `Fetched` |
| A task's progress and end | the task's handle | `Event::TaskProgress`, `Event::TaskDone` |
| A transfer's outcome | `onTransferDone(state, id, outcomes)` | `Event::TransferDone { id, outcomes }` |
| The tab title | `title(state)` | `fn title(&self)` |
| Kept across restarts | `save(state)`, any JSON value | `fn save(&self)`, bytes |
| Data every page shares | `tessoa.storage` | `storage` |

## Testing a page

A page scenario opens the page and plays the user, one step at a time, and
checks the page after each step; `tessoa-plugin test` runs it like any other
scenario. The steps and checks are in [testing.md](testing.md#page-scenarios):
clicks and keys, choosing options, setting values, selecting and opening rows,
answering pickers, waiting for tasks.

```toml
name = "Adds by button and by key"

[[steps]]
open-page = "calculator"

[[steps]]
click = "k1"
expect-text = { key = "display", text = "1" }

[[steps]]
click = "add"

[[steps]]
key = "3"

[[steps]]
key = "Enter"
expect-text = { key = "display", text = "4" }
```

`tessoa-plugin render` prints the page as tessoa receives it, one line per
node: its kind, what it shows, its key, and every attribute that is not the
default. `--steps tests/scenarios/<file>.toml` prints it after the steps of a
scenario. The calculator, freshly opened:

```
page 1 calculator: "Calculator"
keys: 0 1 2 3 4 5 6 7 8 9 . + - * / = Enter Backspace Escape Delete Ctrl+c Meta+c
state: "{\"display\":\"0\",\"acc\":null,\"op\":null,\"fresh\":true}"
column pad=m gap=s
  text "" key=pending size=caption tone=dim align=end
  text "0" key=display size=display align=end
  row gap=s
    button "C" key=clear grow
    button "⌫" key=back grow
    button "÷" key=div grow
    button "×" key=mul grow
  row gap=s
    button "7" key=k7 grow
    button "8" key=k8 grow
    button "9" key=k9 grow
    button "−" key=sub grow
  row gap=s
    button "4" key=k4 grow
    button "5" key=k5 grow
    button "6" key=k6 grow
    button "+" key=add grow
  row gap=s
    button "1" key=k1 grow
    button "2" key=k2 grow
    button "3" key=k3 grow
    button "=" key=eq kind=primary grow
  row gap=s
    button "0" key=k0 grow
    button "." key=dot grow
    button "Copy" key=copy grow
```

The Rust calculator prints the same lines; only the state differs, since
each language saves it its own way. A text field prints the value the page
gives it, and `holds="..."` when it keeps what the user typed instead. The
controls have one line each, written the way the attributes are:

```
    dropdown key=format value=iso options=[iso "2026-10-15", us "10/15/2026", eu "15.10.2026"] tip="How the summary writes the day"
    radio-group key=size value=m options=[s "Small", m "Medium", l "Large"]
    switch "Dark mode" key=dark tip="Only this page"
      number-input key=width value=1920 min=0 max=10000 suffix=px
      slider key=volume value=40 tip="From 0 to 100"
    progress key=busy indeterminate tip="A bar without a value moves"
      color-picker key=color value=#3b82f6
      date-picker key=day value=2026-10-15 min=2026-01-01 max=2026-12-31
      rich-text "Rich text has **bold**, *italic* and `code` spans, <span tone=accent>colours</span>, a link to [a website](https://example.com/), which tessoa asks about first, and a link the page handles: [more options](#more)." key=about wrap grow
```

and a list, a table or a tree has the rows on its screen below it; see
[testing.md](testing.md#looking-at-a-page).

## The manifest

Each kind of page is a `[[pages]]` entry in `plugin.toml` with an `id`, a
`title`, optionally an `icon`, `single` and `where`; the plugin's `[theme]`
colours all its pages. See [manifest.md](manifest.md). The `id` is the kind
that `tessoa.pages.define("<id>", ...)` or `export_pages!(T => "<id>")`
names. A plugin may have pages and no commands.
