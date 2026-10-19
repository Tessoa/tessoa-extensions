# Testing a plugin

Tests run the plugin in the real runner against a scripted host: a folder
prepared from fixture files, a list of answers for the dialogs, and a list of
things that must be true afterwards. A page is tested with steps that play
the user: open the page, click, type, choose, select rows, drop, answer,
restart, and check the page after each step ([Page scenarios](#page-scenarios)).
Steps also wait for the plugin's tasks, change its settings and look at its
column. Requests go nowhere: the scenario answers them ([`[[net]]`](#net)).
No tessoa is needed, and the same test files work for a plugin written in
Rust or in TypeScript.

```
tessoa-plugin test
```

reads every `tests/scenarios/*.toml` in the project, builds the plugin if
needed and runs each scenario. Fixture files live in `tests/fixtures`.

## A scenario

```toml
name = "Renames photos after their capture date"
command = "rename-by-date"

[[files]]
from = "photos/IMG_0001.jpg"

[[files]]
from = "photos/noexif.jpg"
modified = "2025-01-15T08:05:00Z"

[[files]]
from = "photos/notes.txt"
selected = false

[[dialogs]]
kind = "confirm-list"
title = "Rename by capture date"
items = [
  "IMG_0001.jpg  ->  2026-10-03_1422.jpg",
  "noexif.jpg  ->  2025-01-15_0805.jpg",
]
answer = "ok"

[expect]
files = ["2025-01-15_0805.jpg", "2026-10-03_1422.jpg", "notes.txt"]
notifications = [{ title = "Rename by capture date", body = "Renamed 2 of 2 files." }]
log-contains = ["IMG_0001.jpg -> 2026-10-03_1422.jpg"]
```

### Top level

| Key | Meaning |
|---|---|
| `name` | Shown in the report. |
| `command` | The command id to run, as declared in `plugin.toml`. A scenario for a page has `[[steps]]` instead; see [Page scenarios](#page-scenarios). |
| `static-audit` | Default `true`. Set to `false` to run a plugin whose component fails the import audit; useful only for testing the checks that happen at call time. |
| `answer-delay-ms` | Wait this long before answering each dialog. |
| `pick-files`, `pick-folder`, `answer-conflict` | For a scenario with a command: what the user answers when the command, or a task it starts, opens a picker or creates a file whose name is taken. See [Answers](#answers). |

After a command, the tasks it started run to their end before `[expect]` is
checked; each must end done unless `[expect] tasks` says how it ends.

### `[[files]]`

Each entry copies a fixture into a fresh temporary folder. Every entry is part
of the selection unless `selected = false`.

| Key | Meaning |
|---|---|
| `from` | Path under `tests/fixtures`, with `/`. |
| `as` | Name inside the folder. Default: the fixture's own name. |
| `modified` | Modification time to set, `YYYY-MM-DDTHH:MM:SSZ` (UTC). |
| `selected` | Default `true`. |

### `[[dialogs]]`

One entry per dialog the plugin is expected to show, in order: those of
the commands and those of the tasks. The scenario fails if the plugin shows
more or fewer. Every key except `answer` is optional and is only checked when
present.

| Key | Meaning |
|---|---|
| `kind` | `confirm` or `confirm-list`. |
| `title` | Exact title. |
| `body-contains` | A substring of the body. |
| `items` | The exact list of items of a `confirm-list`. |
| `items-count` | The number of items. |
| `answer` | `ok` or `cancel`. |

### `[expect]`

Everything is optional. A missing key is not checked, with two exceptions:
when `denials` is absent, no call may have been refused, and when `tasks` is
absent, every task the command started must have ended done.

| Key | Meaning |
|---|---|
| `result` | How the command ended: `ok` (default), `plugin-error`, `cpu-burst`, `cpu-budget`, `memory-limit`, `trap`, `unknown-command`, or `load-error` when the plugin must be refused before it runs. |
| `error-contains` | A substring of the error text. |
| `files` | The exact contents of the folder afterwards, in any order. |
| `notifications` | Every notification, in order; each with optional `title`, `body`, `body-contains`. |
| `log-contains`, `log-not-contains` | Substrings that must, or must not, appear in some log line. What the plugin prints to standard output and error is part of the log. |
| `denials` | Every call the runner refused, in order; each with optional `call` (for example `files.read`) and `reason-contains`. |
| `tasks` | How the tasks the command started end, each checked as `expect-task` checks a task: `[{ task = 1, state = "failed", reason-contains = "no folder" }]`, `[{ count = 2 }]`. When `tasks` is absent, every task the command started must end done. |

### `[limits]`

`memory-mib`, `cpu-burst-ms`, `cpu-budget-ms`. Defaults are the runner's.
`wait-ms`: how long the steps wait for tasks and the column, default 30000.

### `[settings]`

The values the user set in the plugin's settings before the scenario, by id:
`true` or `false` for a flag, a number, an option's id for a choice, a text.
A value is held to the setting's rules as tessoa holds it; an id the manifest
does not declare fails the scenario.

```toml
[settings]
format = "us"
size = 5
```

### `[[net]]`

The answers to the plugin's requests. Nothing goes out: every request that
passes the runner's checks (the address is on a website the manifest lists,
the headers and the sizes are allowed) is answered by the first entry that
fits, one hop at a time; a redirect an entry answers is followed and checked
like one from a website. A request no entry answers fails the step it was
made in, naming the request.

| Key | Meaning |
|---|---|
| `url` | The whole address, with the query; ending with `*`, the start of addresses. |
| `method` | Only requests of this method. Default: any. |
| `status` | Default 200. |
| `headers` | A table of headers, such as `{ "content-type" = "application/json" }`. |
| `body` | The body as text; or `body-file`, a file under `tests/fixtures`. |
| `error` | Instead of an answer: `timeout`, `too-large`, `failed: <why>` or `not-allowed: <why>`. |

```toml
[[net]]
url = "https://api.example.com/v1/rates?base=EUR"
body = '{"USD": 1.08}'

[[net]]
url = "https://api.example.com/v1/*"
error = "timeout"
```

## What the host does

- Renames happen inside the temporary folder. A target that already exists
  fails the item, which stands in for tessoa's conflict handling.
- The plugin gets exactly the permissions its manifest declares. There is no
  way to grant more from a scenario.
- When the script runs out of answers, further dialogs are cancelled and the
  scenario fails.
- A picked folder is a folder of the scenario's folder, made when it is not
  there. New files are created in it; a name that is taken asks the step's
  `answer-conflict`, and keeping both adds a number the way tessoa does
  (`report (2).csv`).
- Tasks start at once, up to four at a time, and run only while a step waits
  for them (`wait-tasks`, `cancel-task`, and after a command). Between steps
  they stand still, so that every step sees what its own action did. What a
  task reports reaches the plugin as in tessoa: at most five times a second,
  so a page may show a task's last report before the one it ended on.
- Web addresses the plugin offers and the settings it opens are recorded, and
  the offer is shown at once.

## Page scenarios

A scenario for a page has `[[steps]]` instead of `command`: what the user
does, one step at a time, and what the page must show after each step.

```toml
name = "Deleting asks first, and the other page keeps working"

[[steps]]
open-page = "board"

[[steps]]
type = { into = "new", text = "Write report" }

[[steps]]
submit = "new"
expect-children = { key = "col-todo", keys = ["title-todo", "c1"] }

[[steps]]
menu = { on = "card-c1", item = "delete" }
expect-dialog = { title = "Delete card", body-contains = "Write report" }

[[steps]]
answer = "ok"
expect-children = { key = "col-todo", keys = ["title-todo"] }
```

A step does at most one thing and may check several; the checks run after
the action. The first step that fails ends the scenario, and the report
shows every open page as `tessoa-plugin render` prints it. A step refuses
what tessoa would not deliver: a click on a disabled button, on a button that
has a menu (the click opens the menu; use `menu`) or on a node that takes no
click, a key nothing takes, a line break typed into a field of one line, an
option that is disabled, a row that is not on the screen, a node behind an
open popup, a drop on a zone that does not take that payload. A scenario that
passes therefore describes something a user can do.

`[[files]]` makes the folder the user drags files from and selects files in.
`[[dialogs]]` answers the dialogs of the commands the steps run and of the
tasks they start. `[expect]`
is checked after the last step, as for a command. Its `result` and
`error-contains` belong to a command; in a page scenario only
`result = "load-error"` works, for a plugin that must be refused before it
runs.

### Pages

Pages are numbered in the order they open in the scenario, whether a step or
the plugin opened them. A step acts on the page that opened last; `page = 2`
in a step acts on the second page, and so do the steps after it, until
another page opens or is chosen. A step can consist of `page = 2` alone.

`open-page` opens a new page every time, as another window would, even when
the kind is `single`. A page the plugin opens with `pagesHost.open` follows
tessoa's rule instead: a `single` page that is open already comes to the
front, and no second one opens.

### Actions

| Key | What the user does |
|---|---|
| `open-page = "calculator"` | Opens a page of this kind. |
| `run = "add-to-shelf"` | Runs a command on the files of `[[files]]` that are selected. |
| `click = "k7"` | Clicks the button (or icon button) with this key, or turns a checkbox or a switch over, or opens or closes a fold by its title. A button with a menu takes `menu` instead. |
| `key = "Ctrl+c"` | Presses a key, written as a page declares it (`ArrowDown`, `Shift+Tab`): the control that has the focus takes it when it takes such keys (see [Keys and the focus](#keys-and-the-focus)), the page when it declares it. Escape closes the popup that is open. |
| `type = { into = "new", text = "Write report" }` | Selects everything in the text field and types the text, one character at a time: the page gets an input event after each character, and the field holds the text whatever value the page gives it, until the page changes that value. A line break (`\n`) is a new line in a field of several lines, and refused in a field of one line. Typing does not submit. |
| `submit = "new"` | Presses Enter in the field, or Ctrl+Enter in a field of several lines. The page gets the field's text. |
| `toggle = "check-1"` | Clicks the checkbox or the switch: it changes to the opposite of what the page shows. |
| `menu = { on = "card-c1", item = "rename" }` | Chooses an item of the node's menu. On a list, `row = "r3"` right-clicks that row first, which selects it unless it is selected. |
| `choose = { on = "fmt", option = "us" }` | Chooses an option of a dropdown, a radio group, segmented buttons or tabs. An option that is disabled, or chosen already, is refused. |
| `set = { on = "n", value = 20 }` | Sets a number input or a slider to a number, which is put on the control's step and in its range as tessoa does; a colour picker to `"#rrggbb"`; a date picker to `"YYYY-MM-DD"`, or `""` to clear it. A day the calendar does not offer is refused. |
| `drag-split = { on = "s", ratio = 0.5 }` | Drags the bar of a split and lets go. tessoa keeps where it is until the page gives another ratio. |
| `click-link = { on = "r", link = "more" }` | Clicks a link of rich text, named by its action, its address or its text. An action goes to the page; a web address goes to tessoa, which asks the user (check it with `expect-open-url`). |
| `dismiss = "pop"` | Closes the popup with this key: Escape, a click outside it, its close button. |
| `click-row = { on = "results", row = "r12" }` | Clicks a row on the screen of a list, a table or a tree; `shift = true` and `ctrl = true` add the keys. See [Lists](#lists). |
| `activate-row = { on = "results", row = "r12" }` | Opens a row: a double click. |
| `check-row = { on = "t", row = "r1", column = "done" }` | Clicks the checkbox of a row's cell in that column. |
| `row-action = { on = "t", row = "r1", action = "open" }` | Clicks a button at the end of a row. |
| `scroll = { on = "results", row = 6000 }` | Scrolls the list so that this row, counted from 0, is at the top of the screen. |
| `expand = { on = "outline", row = "docs" }` | Opens a row of a tree, or closes it; `open = true` or `false` says which. |
| `sort = { on = "t", column = "size" }` | Clicks the header of a sortable column: first ascending, then the other way round. |
| `focus = "vol"` | Moves the keyboard focus to a control, or to a link of rich text (`"r#2"`). |
| `wait-tasks = true` | Waits until every task of the plugin has ended, at most `[limits] wait-ms`. |
| `cancel-task = 2` | Cancels a task, by its number (from 1, in the order the plugin started them) or its title, and waits until it has stopped. |
| `set-setting = { id = "format", value = "us" }` | Changes one of the plugin's settings in tessoa. |
| `drop-files = { onto = "zone", files = ["a.txt"] }` | Drops files of the scenario's folder on a drop zone that takes files. |
| `drop-item = { onto = "col-done", payload = "c3", before = "c1" }` | Drags the draggable node with this payload onto a drop zone that takes items, before the zone's child with the key `before`; without `before`, to the end. |
| `answer = "ok"` | Answers the earliest question of the page that is still waiting: `ok` or `cancel`. |
| `pick = "ok"` | Answers what tessoa shows for the earliest copy, move or move to the recycle bin the page started: the folder picker, or the confirmation. `"ok"`: every file is done. `"cancel"`: every file is skipped. A list such as `["done", "failed: in use"]`: these outcomes, one per file in order (`done`, `skipped: <why>`, `failed: <why>`). A transfer a command started is picked the same way and reports to no page. |
| `activate = false` | The page's tab stops being the visible one, or becomes it again with `true`. |
| `restart = true` | tessoa quits and starts again. The plugin starts from scratch, with its storage and file references; every page that was open opens again with the state its last render saved. |
| `expect-error = "..."` | Not an action: the step's action must fail with an error that contains this text. Without it, an error the plugin reports fails the step. |

Nothing is copied, moved or deleted: tessoa would do that work, and the
plugin only sees the outcomes. A file reported moved to the recycle bin reads
as gone in `refs.info` afterwards.

An open popup covers the rest of its page: a step that acts on a node
outside it is refused until the popup is closed. Neither can a step reach a
node inside a closed fold or popup, which tessoa does not draw.

### Answers

A picker the plugin opens, and a new file whose name is taken, are answered
by the step whose action leads to them:

| Key | What the user answers |
|---|---|
| `pick-files = ["a.csv"]` | Picks these files of the scenario's folder in the file picker; `[]` cancels. A file the picker does not offer (another extension, or a second file when it takes one) is refused. |
| `pick-folder = "out"` | Picks this folder of the scenario's folder, made when it is not there; `"."` is the folder itself, `false` cancels. |
| `answer-conflict = "keep-both"` | A file of that name exists: keeps both, the new one numbered, or `"skip"`s it. It answers every conflict of the step. |

```toml
[[steps]]
click = "export"
pick-folder = "out"
expect-created = ["out/report.csv"]
```

A command and a task wait for the answer, so it must be in the step that
runs the command or waits for the task; without one the step fails. A page
does not wait: its picker stays open, and a later step can answer it with the
answer alone. An answer no picker takes fails the step too.

### Lists

A list, a table or a tree shows 20 rows on its screen, from the top until a
step scrolls it. The test host asks the page for those rows, as tessoa does;
a step can click only the rows on the screen, and `expect-rows` reads them.
tessoa holds the selection: a click selects a row alone, Ctrl and a click
adds or removes one, Shift and a click selects the rows from the last one
clicked; the page hears `select` with the selected ids in the order of the
rows whenever they change. A selection the page gives replaces the user's
when it differs from the one its previous render gave.

### Keys and the focus

Tab and Shift+Tab move the focus through the controls with a key, in the
order of the page, round from the end to the start; while a popup is open,
only through the controls inside it. The arrow keys are written as a page
declares them, `ArrowUp`, `ArrowDown`, `ArrowLeft` and `ArrowRight`, with
`Shift+` or `Ctrl+` in front when they have them; for the control with the
focus `Up`, `Down`, `Left` and `Right` work as well, as below. Typing into a text field puts the focus
in it, and so does setting a number input or a slider, or clicking a row.
The control with the focus takes its own keys: Enter and Space press a
button, Space turns a checkbox or a switch over, Enter and Space open or
close a fold, Enter opens a link, the arrows move a radio group (Up and Down)
and segmented buttons or tabs (Left and Right) to the next option, Up, Down,
PageUp and PageDown step a number input, the arrows, PageUp, PageDown, Home
and End move a slider, and a list takes the arrows, PageUp, PageDown, Home,
End, Space, Ctrl+A and Enter (with Shift to select a range, with Ctrl to move
only the focus) and, in a tree, Left and Right to close and open rows. A text
field with the focus takes every key except Tab. Other keys go to the page
when it declares them; Tab goes to the page only when it declares Tab and no
control has the focus.

On a button or an icon button with a menu, Enter and Space open the menu
without sending a click or a page key. The menu starts with no item highlighted.
Up and Down (also ArrowUp and ArrowDown), Home and End move among enabled
items; Enter or Space chooses the highlighted item. Escape or another key
closes the menu and is consumed, without reaching the page or closing a popup
behind it. Changing pages or restarting closes the menu as well.

File references are numbered 1, 2, 3 and so on, in the order the plugin
first receives each file, dropped or kept by a command; a file keeps its
number until the plugin releases it. A shelf that keys its rows by reference
has the rows `row-1`, `row-2` for the first two files dropped on it.
`tessoa-plugin render --steps` shows the numbers.

### Checks

`expect-text`, `expect-tree-contains`, `expect-children`, `expect-storage`,
`expect-value`, `expect-rows`, `expect-selected`, `expect-task` and
`expect-column` take one table, or a list of tables to check several.
`expect-transfer`, `expect-dialog` and `expect-request` take one table;
`expect-title`, `expect-clipboard`, `expect-focus`, `expect-open-url`,
`expect-log` and `expect-error` take one string, `expect-created` a list of
strings, `expect-open-settings` `true` or `false`.

| Key | Holds when |
|---|---|
| `expect-text = { key = "display", text = "15" }` | The node shows exactly this text; `contains = "..."` instead of `text` checks a part. A text shows its text; a button, an icon button or a checkbox its label; a text field what it holds, as tessoa keeps it: what the user typed, until the page gives the field another value than the render before (so a page that always gives `value=""` is not emptied by a submit); a file row its file's name; a node that holds others the texts inside it, one per line. |
| `expect-tree-contains = { kind = "file-row", count = 3 }` | Nodes match the description: at least one, or exactly `count`; `count = 0` checks that none does. Every key given must hold: `key`, `kind`, `text`, `contains`, `in` (the key of a node it is inside), `disabled`, `checked`, `selected`, `multiline`, `file` (the file name a file row shows), `tip` (its tooltip). |
| `expect-children = { key = "col-todo", keys = ["title-todo", "c1"] }` | The node's children have these keys, in this order; children without a key are left out. |
| `expect-title = "Shelf (3)"` | The page's tab shows this title. |
| `expect-storage = { key = "board", json = { next = 2 } }` | The plugin's storage holds this under the key: `json` compares the value read as JSON, `text` the value read as text, `contains` a part of that text; `absent = true` checks that the key is not there. TOML has no `null`; check a value that contains one with `contains` or `text`. |
| `expect-clipboard = "42"` | The plugin put this text on the clipboard, since the scenario began or the previous `expect-clipboard`. |
| `expect-transfer = { kind = "move", files = ["a.txt"] }` | The plugin's latest copy, move or move to the recycle bin (`kind` is `copy`, `move` or `trash`) has these files in this order, and it started since the scenario began or the previous `expect-transfer`. |
| `expect-dialog = { title = "Delete card", body-contains = "Write" }` | The page's earliest question that waits is like this: `kind` (`confirm` or `confirm-list`), `title`, `body-contains`, `items`, `items-count`. |
| `expect-value = { key = "fmt", value = "us" }` | The control shows this value: the id of the option chosen in a dropdown, a radio group, segmented buttons or tabs; `true` or `false` for a checkbox, a switch, a fold (open) or a popup (open); a number for a number input, a slider, a split or a progress bar (`"indeterminate"` when it has none); `"#rrggbb"` for a colour; `"YYYY-MM-DD"`, or `""` for none, for a date; the text a text field holds. A control shows the user's change until the page's next tree. |
| `expect-rows = { on = "results", ids = ["r0", "r1"] }` | The rows on the screen of a list, a table or a tree, from its top or from row `from`: `ids`, `texts` (the text of the first cell), `cells` (every cell, as `render` writes a cell that is not plain text); `count` checks how many rows it has. |
| `expect-selected = { on = "results", ids = ["r3"] }` | The selected rows, in the order of the rows. |
| `expect-focus = "vol"` | This control, or this link (`"r#2"`), has the keyboard focus; `""` for none. |
| `expect-task = { task = 1, state = "done" }` | A task, by `task` (its number) or `title`, the latest without either, is `running`, `done`, `failed`, `cancelled` or `stopped` (tessoa quit while it ran); `reason-contains` checks why it failed; `count` how many tasks the plugin started. |
| `expect-request = { method = "POST", url = "https://api.example.com/v1/x" }` | The plugin's latest request since the scenario began or the previous `expect-request`: `method`, `url`, `headers` (a table, names in any case), `body`, `body-contains`. |
| `expect-open-url = "https://example.com/"` | The latest web address offered since the scenario began or the previous `expect-open-url`, by the plugin or by a link of rich text. |
| `expect-open-settings = true` | The plugin opened its settings since the scenario began or the previous check. |
| `expect-created = ["out/report.csv"]` | The files the plugin created since the scenario began or the previous `expect-created`, in order, as paths in the scenario's folder. |
| `expect-column = { column = "lines", files = ["a.txt"], values = [3] }` | Asks the plugin's column for the values of these files of the folder (default: every file, by name), as tessoa does for the rows of its file list, and checks them; a file the column's `when` leaves out shows nothing, and the plugin is not asked for it. A value is a text, a number, `""` for none, `{ size = 1024 }`, `{ date = "2026-10-15T08:00:00Z" }`, `{ text = "1920×1080", key = 2073600 }` for a text that sorts by a number. |
| `expect-log = "fetched"` | A line of the log since the scenario began or the previous `expect-log` contains this. |

`expect-text`, `expect-tree-contains`, `expect-children`, `expect-title`,
`expect-dialog`, `expect-value`, `expect-rows`, `expect-selected` and
`expect-focus` look at the page the step acts on.

### Looking at a page

`tessoa-plugin render` prints a page as text, one line per node; see
[pages.md](pages.md#testing-a-page). `tessoa-plugin render --steps
tests/scenarios/board.toml` prints the pages after the steps of a scenario,
which is the quickest way to see what a check should say.

A list, a table or a tree has the rows on its screen below it, one level
deeper, and a line that counts the rows not shown; `--rows 100..140` shows
those rows instead:

```
  table key=files count=2400 columns=[name "Name" fill sortable, size "Size" 6em end sortable] sort=size:desc
    f17 "report.pdf" | "2.1 MB"(dim) actions=[open eye "Open"] selected
    f3 "photo.jpg" | "1.8 MB"(dim) actions=[open eye "Open"]
    … 2398 more rows
```

A cell is `"text"`, with `(dim)` or `(icon=eye)` after it when it has them,
`file:"a.txt"`, `image:images/a.png` or `image:added=3`, `[x]` or `[ ]`,
`progress=0.5`, or `-` when it is empty. A row of a tree is indented by its
depth and starts with `[+]` when it can open, `[-]` when it is open. What
tessoa keeps besides the tree follows a node's attributes: `holds="..."` for
the text a field holds, `holds=0.4` for where the bar of a split is, and
`focused` on the control with the focus. A failed scenario shows the open
pages this way, and the plugin's tasks below them.

## Unit tests

Logic that does not touch the host is tested the usual way: `cargo test` for a
Rust plugin, the project's own test runner for a TypeScript plugin. The
example keeps its EXIF reader in a module with native unit tests for that
reason.

## From Rust

The `tessoa-plugin-testhost` crate exposes the pieces: `ScriptedHost`
implements the runner's `Host` trait and records everything, also for the
plugin's workers (`absorb` takes in what they recorded), `Sandbox` builds the
folder, `run_scenario` runs a parsed scenario, `render_pages` writes pages as
text and `run_command_and_tasks` runs a command and then its tasks. The
crate's own tests use them for checks a scenario cannot express, such as a
crash in one session leaving another untouched.
