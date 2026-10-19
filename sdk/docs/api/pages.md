<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# pages

What a plugin with pages exports. A page is a whole tab, or a section of
the sidebar: the plugin describes its content as a tree of nodes, tessoa
draws it with its own widgets, and tells the plugin what the user did.
The description is the whole page every time; tessoa compares it with
the previous one and keeps what the user typed, scrolled and focused,
matched by node keys.

A control shows the value the page gives it. When the user changes it,
tessoa sends an event and shows the new value until the page has handled
the event; then it shows what the page renders, so a page that keeps the
old value puts the control back. A text field is the exception: it keeps
what the user typed until the page gives it another value.

A plugin with pages exports this interface: it is built for the world `tessoa-plugin-pages`. The runner calls these functions; the plugin never calls them itself. In Rust, turn on the SDK's `pages` feature, implement `tessoa_plugin_sdk::pages::Page` for each kind of page and pass the types to `tessoa_plugin_sdk::export_pages!`. In TypeScript, call `tessoa.pages.define(kind, page)` for each kind when the script loads; `render` returns JSX. Every script plugin exports this interface; a script without pages answers that it has none.

## Functions

### open

```
wit  open: func(page: page-id, kind: string, state: option<list<u8>>, args: option<list<u8>>) -> result<_, string>
rust fn open(page: PageId, state: Option<Vec<u8>>, args: Option<Vec<u8>>) -> Self   (in impl Page)
ts   open({ page, state, args }): S   (in tessoa.pages.define; state and args parsed from JSON)
```

A page opens: newly, when tessoa restores it after a restart, or when
it is reloaded after the plugin stopped. `state` is what the last
render returned; `args` are what `pages-host.open` passed. An error
is shown in place of the page.

### event

```
wit  event: func(page: page-id, ev: page-event) -> result<_, string>
rust fn event(&mut self, event: Event) -> Result<(), String>   (in impl Page)
ts   onClick, onInput, onSubmit, onToggle, onChange, onLink, onDismiss, onDropFiles, onDropItem, onMenu on elements, and onSelect, onActivate, onSort, onExpand, onRowCheck, onRowAction on lists, tables and trees; onKey, onTransferDone, onActivated in tessoa.pages.define; the answers to pickers, new files and requests settle the promises their await waits for
```

Something happened on the page. An error goes to the plugin log; the
page stays open.

### render

```
wit  render: func(page: page-id) -> result<tree, string>
rust fn render(&self) -> Node   (in impl Page, with title, keys and save)
ts   render(state: S): JSX element   (in tessoa.pages.define, with title, keys and save)
```

Describe the page. Called after `open` and after calls that
invalidated the page. An error, or a tree that breaks the rules of
`tree`, closes the page and shows the message in its place.

### close

```
wit  close: func(page: page-id)
rust the page value is dropped
ts   close?(state: S): void   (in tessoa.pages.define)
```

The page closed. Its id is not used again for this plugin.

### rows

```
wit  rows: func(page: page-id, list: string, start: u32, count: u32) -> result<list<row>, string>
rust fn rows(&self, list: &str, start: usize, count: usize) -> Vec<Row>   (in impl Page)
ts   row={(i) => row} on <list>, <table> and <tree>
```

Rows of the list, table or tree with the key `list`: up to `count`
rows from row `start`, fewer at the end. tessoa asks for the rows on
screen and a screen above and below them, at most 2000 at a time, at
most 1 MiB. Describe each row from the page's data; the same data
gives the same row.

### row-ids

```
wit  row-ids: func(page: page-id, list: string, start: u32, count: u32) -> result<list<string>, string>
rust fn row_ids(&self, list: &str, start: usize, count: usize) -> Vec<String>   (in impl Page; by default the ids of rows)
ts   rowId={(i) => id} on <list>, <table> and <tree>   (optional; by default the ids of row)
```

The ids of rows, without the rest of them: for selecting a range of
rows, or all rows, the user has not scrolled to.

## Types

### page-id

Defined in [pages-host](pages-host.md#page-id).

### file-ref

Defined in [files](files.md#file-ref).

### folder

Defined in [files](files.md#folder).

### create-outcome

Defined in [files](files.md#create-outcome).

### error

Defined in [files](files.md#error).

### transfer-outcome

Defined in [refs](refs.md#transfer-outcome).

### response

Defined in [net](net.md#response).

### net-error

`error`, defined in [net](net.md#error).

### menu-item

Record; Rust `MenuItem`, TypeScript `MenuItem`.

| Field | Type | Meaning |
|---|---|---|
| `id` | `string` |  |
| `label` | `string` |  |
| `disabled` | `bool` |  |
| `danger` | `bool` | Drawn in the warning colour, for destructive choices. |
| `separator-after` | `bool` | A separator line follows this item. |

### size-hint

Variant; Rust `SizeHint`, TypeScript `SizeHint`.

| Case | Payload | Meaning |
|---|---|---|
| `auto` | none | As large as the content. |
| `fill` | none | As large as the parent allows. |
| `em` | `f32` | A fixed size in units of the current font size. |

### space

Enum; Rust `Space`, TypeScript `Space`.

| Case | Meaning |
|---|---|
| `none` |  |
| `xs` |  |
| `s` |  |
| `m` |  |
| `l` |  |

### align

Enum; Rust `Align`, TypeScript `Align`.

| Case | Meaning |
|---|---|
| `start` |  |
| `center` |  |
| `end` |  |
| `stretch` |  |

### justify

Enum; Rust `Justify`, TypeScript `Justify`.

| Case | Meaning |
|---|---|
| `start` |  |
| `center` |  |
| `end` |  |
| `between` |  |

### layout

Record; Rust `Layout`, TypeScript `Layout`.

How a node takes space inside its parent and arranges its children.

| Field | Type | Meaning |
|---|---|---|
| `grow` | `bool` | Take a share of the length the parent has left along its direction. Children that grow get equal shares whatever their content, as with CSS `flex: 1 1 0`, but none gets less than its content needs. |
| `w` | `size-hint` |  |
| `h` | `size-hint` |  |
| `pad` | `space` | Space inside the node, around its children. |
| `gap` | `space` | Space between the children. |
| `align` | `align` | Placement of the children across the parent's direction. |
| `justify` | `justify` | Placement of the children along the parent's direction. |
| `wrap` | `bool` | Children that do not fit continue on the next line. |
| `scroll` | `bool` | The node scrolls when its children do not fit. |

### text-size

Enum; Rust `TextSize`, TypeScript `TextSize`.

| Case | Meaning |
|---|---|
| `caption` |  |
| `body` |  |
| `title` |  |
| `display` |  |

### text-align

Enum; Rust `TextAlign`, TypeScript `TextAlign`.

| Case | Meaning |
|---|---|
| `start` |  |
| `center` |  |
| `end` |  |

### tone

Enum; Rust `Tone`, TypeScript `Tone`.

| Case | Meaning |
|---|---|
| `normal` |  |
| `dim` |  |
| `accent` |  |
| `danger` |  |
| `success` |  |

### button-kind

Enum; Rust `ButtonKind`, TypeScript `ButtonKind`.

| Case | Meaning |
|---|---|
| `normal` |  |
| `primary` |  |
| `danger` |  |
| `quiet` | No frame until hovered. |

### accepts

Enum; Rust `Accepts`, TypeScript `Accepts`.

What a drop zone takes.

| Case | Meaning |
|---|---|
| `files` | Files dragged from tessoa or from other programs. |
| `items` | Draggable nodes of the same page. |
| `both` |  |

### text-props

Record; Rust `TextProps`, TypeScript `TextProps`.

| Field | Type | Meaning |
|---|---|---|
| `text` | `string` |  |
| `size` | `text-size` |  |
| `tone` | `tone` |  |
| `wrap` | `bool` | Break lines to fit the width instead of cutting the text off. |
| `align` | `text-align` |  |

### button-props

Record; Rust `ButtonProps`, TypeScript `ButtonProps`.

| Field | Type | Meaning |
|---|---|---|
| `label` | `string` |  |
| `kind` | `button-kind` |  |
| `icon` | `option<string>` | A name from tessoa's icon set, drawn in the user's theme. Names: `plus`, `close`, `arrow-left`, `arrow-right`, `arrow-up`, `arrow-down`, `chevron-down`, `chevron-right`, `check`, `search`, `menu`, `more`, `reload`, `undo`, `copy`, `cut`, `paste`, `new-folder`, `rename`, `delete`, `play`, `note`, `sliders`, `eye`, `sun`, `gear`, `image`, `keyboard`, `monitor`, `info`, `pin`, `question`, `tag`, `filter`, `puzzle`. Later versions only add names. A name tessoa does not know shows the puzzle piece. |
| `disabled` | `bool` |  |

### input-props

Record; Rust `InputProps`, TypeScript `InputProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `string` |  |
| `placeholder` | `string` |  |
| `disabled` | `bool` |  |
| `multiline` | `bool` | Several lines: Enter starts a new line, Ctrl+Enter submits. |

### check-props

Record; Rust `CheckProps`, TypeScript `CheckProps`.

| Field | Type | Meaning |
|---|---|---|
| `checked` | `bool` |  |
| `label` | `string` |  |

### file-row-props

Record; Rust `FileRowProps`, TypeScript `FileRowProps`.

| Field | Type | Meaning |
|---|---|---|
| `file` | `file-ref` |  |
| `selected` | `bool` |  |
| `draggable` | `bool` | The row can be dragged out as a file. |

### drop-props

Record; Rust `DropProps`, TypeScript `DropProps`.

| Field | Type | Meaning |
|---|---|---|
| `accepts` | `accepts` |  |
| `hint` | `option<string>` | Shown in the zone while nothing is dragged over it. |

### drag-props

Record; Rust `DragProps`, TypeScript `DragProps`.

| Field | Type | Meaning |
|---|---|---|
| `payload` | `string` | Delivered to the drop zone's `drop` event. |

### choice-option

Record; Rust `ChoiceOption`, TypeScript `ChoiceOption`.

One option of a dropdown, a radio group, segmented buttons or tabs.

| Field | Type | Meaning |
|---|---|---|
| `id` | `string` | Names the option in `value` and in `choose` events. Unique within the node. |
| `label` | `string` |  |
| `icon` | `option<string>` | A name from tessoa's icon set, as for `button-props.icon`. |
| `disabled` | `bool` |  |

### choice-props

Record; Rust `ChoiceProps`, TypeScript `ChoiceProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `string` | The id of the chosen option, or an empty string for none. |
| `options` | `list<choice-option>` | At most 256. |
| `disabled` | `bool` |  |

### number-props

Record; Rust `NumberProps`, TypeScript `NumberProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `f64` | Shown within `min` and `max`. |
| `min` | `f64` |  |
| `max` | `f64` |  |
| `step` | `f64` | What the arrows and the arrow keys add or take; a typed number is rounded to it. Greater than 0. |
| `decimals` | `u8` | Digits shown after the decimal point. |
| `suffix` | `string` | Shown dimmed after the number, such as `px` or `%`. |
| `disabled` | `bool` |  |

### slider-props

Record; Rust `SliderProps`, TypeScript `SliderProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `f64` | Shown within `min` and `max`. |
| `min` | `f64` |  |
| `max` | `f64` |  |
| `step` | `f64` | The values the slider stops at, from `min` on. Greater than 0. |
| `vertical` | `bool` | Upright, with `min` at the bottom. |
| `disabled` | `bool` |  |

### progress-props

Record; Rust `ProgressProps`, TypeScript `ProgressProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `option<f32>` | How far, from 0 to 1; none when it is not known, which shows a moving bar. |

### image-src

Variant; Rust `ImageSrc`, TypeScript `ImageSrc`.

| Case | Payload | Meaning |
|---|---|---|
| `file` | `string` | A PNG, JPEG, GIF, WebP, BMP or SVG file in the plugin's directory, written with `/`, such as `images/logo.png`. |
| `added` | `u64` | An image the plugin added with `images.add` or `images.add-rgba`. |

### fit

Enum; Rust `Fit`, TypeScript `Fit`.

How an image fills its node, as CSS `object-fit`.

| Case | Meaning |
|---|---|
| `contain` | Whole, as large as fits, keeping its shape. |
| `cover` | Covering the node, keeping its shape; the rest is cut off. |
| `fill` | Stretched to the node. |
| `none` | At its own size. |

### image-props

Record; Rust `ImageProps`, TypeScript `ImageProps`.

| Field | Type | Meaning |
|---|---|---|
| `src` | `image-src` |  |
| `fit` | `fit` |  |
| `alt` | `string` | Says what the image shows; its tooltip when the node has none. |

### link

Variant; Rust `Link`, TypeScript `Link`.

Where a link of rich text leads.

| Case | Payload | Meaning |
|---|---|---|
| `url` | `string` | A web address, `http` or `https`. tessoa asks the user before it opens it in the browser; the page hears nothing of it. |
| `action` | `string` | Handled by the page: a click sends `link` with this id. |

### span

Record; Rust `Span`, TypeScript `Span`.

A stretch of rich text in one style.

| Field | Type | Meaning |
|---|---|---|
| `text` | `string` |  |
| `bold` | `bool` |  |
| `italic` | `bool` |  |
| `code` | `bool` | In a fixed-width font on a faint background. |
| `tone` | `tone` |  |
| `link` | `option<link>` |  |

### rich-props

Record; Rust `RichProps`, TypeScript `RichProps`.

| Field | Type | Meaning |
|---|---|---|
| `spans` | `list<span>` | At most 2048, with at most 64 KiB of text together. |
| `size` | `text-size` |  |
| `wrap` | `bool` | Break lines to fit the width instead of cutting the text off. |
| `align` | `text-align` |  |

### color-props

Record; Rust `ColorProps`, TypeScript `ColorProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `u32` | `0xrrggbb`. |
| `disabled` | `bool` |  |

### date

Record; Rust `Date`, TypeScript `Date`.

A day of the calendar.

| Field | Type | Meaning |
|---|---|---|
| `year` | `u16` | 1 to 9999. |
| `month` | `u8` | 1 to 12. |
| `day` | `u8` | 1 to the last day of the month. |

### date-props

Record; Rust `DateProps`, TypeScript `DateProps`.

| Field | Type | Meaning |
|---|---|---|
| `value` | `option<date>` | None shows an empty button. |
| `min` | `option<date>` | Days before `min` or after `max` cannot be chosen. |
| `max` | `option<date>` |  |
| `disabled` | `bool` |  |

### selection

Enum; Rust `Selection`, TypeScript `Selection`.

How rows of a list, a table or a tree can be selected.

| Case | Meaning |
|---|---|
| `none` | Not at all; Enter and a double click still send `activate`. |
| `single` |  |
| `multi` |  |

### column

Record; Rust `Column`, TypeScript `Column`.

A column of a table. A list or a tree uses its first column only.

| Field | Type | Meaning |
|---|---|---|
| `id` | `string` | Names the column in `sort` and `row-check` events. Unique within the node, not empty. |
| `title` | `string` | Shown in the table's header. |
| `width` | `size-hint` | The width the column starts with; the user can drag it, and tessoa remembers it. |
| `align` | `text-align` |  |
| `sortable` | `bool` | A click on the header sends `sort`. |

### sort-state

Record; Rust `SortState`, TypeScript `SortState`.

| Field | Type | Meaning |
|---|---|---|
| `column` | `string` | The id of a sortable column. |
| `descending` | `bool` |  |

### list-props

Record; Rust `ListProps`, TypeScript `ListProps`.

| Field | Type | Meaning |
|---|---|---|
| `count` | `u32` | How many rows there are, at most 1,000,000. tessoa asks for the ones on screen with `rows`. |
| `columns` | `list<column>` | At most 32. |
| `selection` | `selection` |  |
| `selected` | `option<list<string>>` | Selects these rows by id, replacing what the user selected, when it differs from what the previous render gave; none leaves the selection to the user. At most 10,000. |
| `sort` | `option<sort-state>` | The column the page sorted its rows by, shown as an arrow in the header; tessoa does not sort. |
| `two-line` | `bool` | Each row has a second, dimmed line: its `detail`. |
| `empty` | `option<string>` | Shown when there are no rows. |

### grid-props

Record; Rust `GridProps`, TypeScript `GridProps`.

| Field | Type | Meaning |
|---|---|---|
| `columns` | `u32` | Children per row, 1 to 32. A column is as wide as its widest child; room to spare is shared equally. |

### fold-props

Record; Rust `FoldProps`, TypeScript `FoldProps`.

| Field | Type | Meaning |
|---|---|---|
| `title` | `string` |  |
| `open` | `bool` | The children show. Closed, they stay in the tree but are not drawn. |

### split-props

Record; Rust `SplitProps`, TypeScript `SplitProps`.

| Field | Type | Meaning |
|---|---|---|
| `ratio` | `f32` | The share of the first child, between 0 and 1. tessoa keeps where the user dragged the bar, until the page gives another ratio. |
| `vertical` | `bool` | One above the other instead of side by side. |

### popup-props

Record; Rust `PopupProps`, TypeScript `PopupProps`.

| Field | Type | Meaning |
|---|---|---|
| `open` | `bool` |  |
| `title` | `option<string>` | A title line with a close button. |
| `anchor` | `option<string>` | The key of a node to show the popup below; none centres it on the page. |
| `width` | `size-hint` |  |

### node-kind

Variant; Rust `NodeKind`, TypeScript `NodeKind`.

| Case | Payload | Meaning |
|---|---|---|
| `column` | none | Children top to bottom. |
| `row` | none | Children left to right. |
| `card` | none | A column with a frame and a background. |
| `divider` | none | A thin line across the parent. |
| `spacer` | none | Empty space; with `grow`, pushes its siblings apart. |
| `text` | `text-props` |  |
| `button` | `button-props` | Sends `click` with the node's key. |
| `icon-button` | `button-props` | A button that shows only its icon; the label becomes its tooltip. |
| `text-input` | `input-props` | Sends `input` on every change and `submit` on Enter, or Ctrl+Enter when it has several lines. |
| `checkbox` | `check-props` | Sends `toggle` with the new state. |
| `file-row` | `file-row-props` | One file, drawn by tessoa: icon, name, size, a dimmed look when the file is gone. Double click opens it, it can be dragged out, and its context menu is tessoa's file menu. |
| `drop-zone` | `drop-props` | Its children are a target for dragged files or items; a drop sends `drop` with the zone's key. |
| `draggable` | `drag-props` | Its children can be dragged onto drop zones of the same page. Give it a key, such as the id of the item it shows: a drop names the draggable it landed in front of by that key, and never names one without a key. |
| `dropdown` | `choice-props` | A button showing the chosen option; clicking it opens the options. Sends `choose`. |
| `radio-group` | `choice-props` | The options one below the other, each with a round mark. Sends `choose`. |
| `segmented` | `choice-props` | The options side by side as joined buttons. Sends `choose`. |
| `tabs` | `choice-props` | A row of tabs. Only the tabs: the page shows what belongs to the chosen one below them. Sends `choose`. |
| `switch` | `check-props` | An on/off switch. Sends `toggle` with the new state. |
| `number-input` | `number-props` | A number field with up and down arrows. Sends `value` on Enter, when it loses the focus, and on the arrows; not while typing. |
| `slider` | `slider-props` | Sends `value` while it is dragged, and once more when it is let go. |
| `progress` | `progress-props` |  |
| `image` | `image-props` |  |
| `rich-text` | `rich-props` | Text with bold, italic and code spans and links. |
| `color-picker` | `color-props` | A swatch with the colour's code; clicking it opens a colour picker. Sends `color`. |
| `date-picker` | `date-props` | A button showing the date; clicking it opens a calendar. Sends `date`. |
| `list` | `list-props` | Rows from `rows`, one column; tessoa asks only for the rows on screen, so a list can be long. Give it a key. |
| `table` | `list-props` | A list with columns under a header that can sort and resize them. |
| `tree` | `list-props` | A list whose rows have levels and can expand; the page decides which rows exist, `rows` describes them with their depth. |
| `grid` | `grid-props` | Its children in a grid, filled row by row. |
| `fold` | `fold-props` | A title that opens and closes the children below it. Sends `toggle` with the new state. |
| `split` | `split-props` | Exactly two children side by side, or one above the other, with a bar between them that the user can drag; tessoa remembers where. Sends `split` when the bar is let go. |
| `popup` | `popup-props` | Its children in a panel over the page; only one popup of a page is open at a time. Escape, a click outside and its close button send `dismiss`; it closes when the page sets `open` to false. |

### node

Record; Rust `Node`, TypeScript `Node`.

| Field | Type | Meaning |
|---|---|---|
| `parent` | `option<u32>` | Position of the parent in `tree.nodes`; none for the root. |
| `key` | `option<string>` | Names the node in events and keeps its state across renders. Unique within the page. Nodes that the user interacts with need one; items of a list should have one. |
| `kind` | `node-kind` |  |
| `layout` | `layout` |  |
| `menu` | `list<menu-item>` | A small menu: right-clicking the node opens it, and so does a left click on a button or an icon button that has one, which then sends no `click`. The choice arrives as a `menu` event. |
| `tooltip` | `option<string>` | A line shown when the pointer rests on the node, at most 1 KiB. An icon button without one shows its label, an image its `alt`. |

### tree

Record; Rust `Tree`, TypeScript `Tree`.

One page, flattened: every node names its parent by position in
`nodes`. The first node is the root and the only node without a
parent; a parent always comes before its children, and children
appear in their order on screen. At most 4096 nodes, at most 64
levels deep (the root is level 0, its children level 1), and 1 MiB.

| Field | Type | Meaning |
|---|---|---|
| `title` | `option<string>` | The title on the page's tab or sidebar section. None keeps the title from the manifest or from `pages-host.set-title`. |
| `keys` | `list<string>` | Keys the page handles while it has the focus and no text field is focused. A key is written the way it types: a character such as `8`, `+` or `a` (`A` with Shift), or one of the names `Enter`, `Escape`, `Backspace`, `Delete`, `Tab`, `Space`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `Home`, `End`, `PageUp`, `PageDown` and `F1` to `F12`. `Ctrl+`, `Alt+` and `Meta+` in front add a modifier, for example `Ctrl+z`; `Shift+` does so in front of a name only, for example `Shift+Tab`. Every other key works as it does elsewhere in tessoa, and so do Ctrl+Shift combinations and Ctrl+Tab, which a page cannot take. |
| `state` | `option<list<u8>>` | Bytes the host keeps for the page and hands back to `open` when tessoa restarts or the page is reloaded. At most 16 KiB; more is dropped and logged. |
| `nodes` | `list<node>` |  |

### expander

Enum; Rust `Expander`, TypeScript `Expander`.

Whether a row of a tree has children, and whether they show.

| Case | Meaning |
|---|---|
| `none` |  |
| `collapsed` |  |
| `expanded` |  |

### cell-text

Record; Rust `CellText`, TypeScript `CellText`.

| Field | Type | Meaning |
|---|---|---|
| `text` | `string` | The first 1 KiB shows; more is cut off. |
| `tone` | `tone` |  |
| `icon` | `option<string>` | A name from tessoa's icon set, before the text. |

### cell

Variant; Rust `Cell`, TypeScript `Cell`.

One cell of a row.

| Case | Payload | Meaning |
|---|---|---|
| `empty` | none |  |
| `text` | `cell-text` |  |
| `file` | `file-ref` | A file issued to the plugin, drawn as in tessoa's file list: it opens on a double click, can be dragged out, and has tessoa's file menu. |
| `image` | `image-src` | A small picture. |
| `check` | `bool` | A checkbox; a click on it sends `row-check`. |
| `progress` | `f32` | A progress bar, from 0 to 1. |

### row-action

Record; Rust `RowAction`, TypeScript `RowAction`.

A button at the end of a row, shown while the pointer is over the
row. A click sends `row-action`.

| Field | Type | Meaning |
|---|---|---|
| `id` | `string` |  |
| `icon` | `string` | A name from tessoa's icon set. |
| `label` | `string` | Its tooltip. |

### row

Record; Rust `Row`, TypeScript `Row`.

One row of a list, a table or a tree. Rows are data, not nodes: they
have neither keys nor layouts, and have the same height.

| Field | Type | Meaning |
|---|---|---|
| `id` | `string` | Names the row in events; the selection follows it when rows move. At most 128 bytes, unique within the list. |
| `cells` | `list<cell>` | One per column, in order. |
| `detail` | `option<string>` | The second line, when the list has `two-line`. |
| `depth` | `u8` | The level in a tree, 0 at the top. Lists and tables ignore it. |
| `expander` | `expander` |  |
| `actions` | `list<row-action>` | At most 4. |
| `tone` | `tone` |  |

### key-press

Record; Rust `KeyPress`, TypeScript `KeyPress`.

| Field | Type | Meaning |
|---|---|---|
| `key` | `string` | The key, written as the page declared it. |
| `ctrl` | `bool` |  |
| `shift` | `bool` |  |
| `alt` | `bool` |  |
| `meta` | `bool` | The Command key on macOS, the Windows key on Windows. |

### drop-payload

Variant; Rust `DropPayload`, TypeScript `DropPayload`.

| Case | Payload | Meaning |
|---|---|---|
| `files` | `list<file-ref>` | Files, already turned into references for this plugin. |
| `item` | `tuple<string, option<string>>` | A draggable node of the page: its payload, and the key of the draggable it was dropped in front of, none at the end. Only the zone's own children that are draggable and have a key count, and never the one being dragged. |

### expand-event

Record; Rust `ExpandEvent`, TypeScript `ExpandEvent`.

The expander of a tree's row was clicked.

| Field | Type | Meaning |
|---|---|---|
| `list` | `string` | The key of the tree. |
| `row` | `string` | The id of the row. |
| `open` | `bool` | True to show its children. |

### sort-event

Record; Rust `SortEvent`, TypeScript `SortEvent`.

A sortable column's header was clicked: first ascending, then the
other way round. The page sorts and renders `sort` to match.

| Field | Type | Meaning |
|---|---|---|
| `list` | `string` | The key of the table. |
| `column` | `string` |  |
| `descending` | `bool` |  |

### row-check-event

Record; Rust `RowCheckEvent`, TypeScript `RowCheckEvent`.

The checkbox of a row's cell was clicked.

| Field | Type | Meaning |
|---|---|---|
| `list` | `string` |  |
| `row` | `string` |  |
| `column` | `string` |  |
| `checked` | `bool` | The new state. |

### row-action-event

Record; Rust `RowActionEvent`, TypeScript `RowActionEvent`.

A button at the end of a row was clicked.

| Field | Type | Meaning |
|---|---|---|
| `list` | `string` |  |
| `row` | `string` |  |
| `action` | `string` | The id of the `row-action`. |

### page-event

Variant; Rust `PageEvent`, TypeScript `PageEvent`.

| Case | Payload | Meaning |
|---|---|---|
| `click` | `string` | A button was pressed. Carries the node key. |
| `submit` | `string` | Enter in a text field. Carries the node key. |
| `input` | `tuple<string, string>` | The text of a field changed. Carries the node key and the new text. |
| `toggle` | `tuple<string, bool>` | A checkbox changed. Carries the node key and the new state. |
| `key` | `key-press` | A declared key was pressed. |
| `drop` | `tuple<string, drop-payload>` | Something was dropped on a drop zone. Carries the zone's key. |
| `transfer-done` | `tuple<u64, list<transfer-outcome>>` | A copy, move or delete the page started has ended. Carries the transfer id and one outcome per file, in order. |
| `dialog` | `tuple<u64, bool>` | The user answered `pages-host.ask`. Carries the dialog id and true for the ok button. |
| `menu` | `tuple<string, string>` | A menu item was chosen. Carries the node key and the item id. |
| `activated` | `bool` | The page came into view (true) or went out of view: as a tab, it became the visible tab of its pane or stopped being it; in the sidebar, its section was shown or hidden. |
| `choose` | `tuple<string, string>` | An option of a dropdown, a radio group, segmented buttons or tabs was chosen. Carries the node key and the option id. |
| `value` | `tuple<string, f64>` | A number input or a slider changed. Carries the node key and the new value, within the node's range and on its step. While a slider is dragged, only the latest value waits to be delivered. |
| `color` | `tuple<string, u32>` | A colour was picked. Carries the node key and `0xrrggbb`; while the picker is dragged, only the latest waits to be delivered. |
| `date` | `tuple<string, option<date>>` | A date was chosen, or cleared. Carries the node key and the date. |
| `split` | `tuple<string, f32>` | The bar of a split was let go. Carries the node key and the share of the first child. |
| `link` | `tuple<string, string>` | A link of rich text with an `action` was clicked. Carries the node key and the action. |
| `dismiss` | `string` | The user closed a popup: Escape, a click outside it or its close button. Carries the node key. The popup stays open until the page sets `open` to false. |
| `select` | `tuple<string, list<string>>` | The selected rows of a list changed. Carries the node key and the ids of the selected rows, in the order of the rows. |
| `activate` | `tuple<string, string>` | A row was opened: double click or Enter. Carries the node key and the row id. |
| `expand` | `expand-event` |  |
| `sort` | `sort-event` |  |
| `row-check` | `row-check-event` |  |
| `row-action` | `row-action-event` |  |
| `picked-files` | `tuple<u64, list<file-ref>>` | The user picked files for `files.pick-files-start`. Carries the call id and references to the files, empty when cancelled. |
| `picked-folder` | `tuple<u64, option<folder>>` | The user picked a folder for `files.pick-folder-start`. Carries the call id and the folder, none when cancelled. |
| `created` | `tuple<u64, result<create-outcome, error>>` | A file of `files.create-file-start` was created, skipped or refused. Carries the call id and the outcome. |
| `fetched` | `tuple<u64, result<response, net-error>>` | The answer to `net.fetch-start`. Carries the request id and the response. |
