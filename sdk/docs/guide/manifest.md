# The plugin manifest

Every plugin has a `plugin.toml` in its root directory. It is the only file an
installer reads before looking at the component, so it carries identity, entry
point, the commands, pages, columns and settings the plugin contributes and
the permissions it needs.

```toml
[plugin]
id = "com.example.rename-by-date"
name = "Rename by capture date"
version = "0.1.0"
api = "tessoa:plugin@0.2.0"
min-host = "0.29.0"

[entry]
kind = "wasm"
file = "dist/plugin.wasm"

[[commands]]
id = "rename-by-date"
title = "Rename by capture date"
where = ["context-menu"]
when = { ext = ["jpg", "jpeg", "heic", "png"], min-selected = 1 }

[permissions]
files = ["read-selected", "rename-selected"]
```

## `[plugin]`

| Key | Required | Meaning |
|---|---|---|
| `id` | yes | Globally unique, reverse-domain style. Lower-case letters, digits, `.`, `_` and `-`; no leading dot, no `..`. It doubles as the directory name once installed. |
| `name` | yes | Shown to the user. |
| `version` | yes | `MAJOR.MINOR.PATCH`, optionally followed by `-label`. |
| `api` | yes | The interface version the plugin was built against, exactly as in the `package` line of `wit/tessoa-plugin.wit`. A runner loads plugins built for its own version or an earlier release of the same line (`0.2.0` with a `0.2.1` runner); it refuses a later version and other lines. |
| `min-host` | no | The oldest tessoa version that may load the plugin. |

## `[entry]`

| Key | Meaning |
|---|---|
| `kind` | `wasm` for a component (Rust and other compiled languages), `js` for a bundled script that the JavaScript engine runs. |
| `file` | Path of the entry file relative to the manifest, with `/` as separator. It must stay inside the plugin directory: no leading `/`, no drive letter, no `..`. |

## `[[commands]]`

One entry per command. A plugin has commands, pages, columns, or several of
them.

| Key | Meaning |
|---|---|
| `id` | Unique within the plugin, same character rules as the plugin id. This is the string `run-command` receives. |
| `title` | The menu text; `title-zh` and the others for other languages (see [Texts in other languages](#texts-in-other-languages)). |
| `where` | Where the command appears. This version knows `context-menu`. |
| `when` | Optional filter: `ext` (extensions without the dot; empty or absent means any file), `min-selected`, `max-selected`. |

## `[[pages]]`

One entry per page: a whole tab the plugin describes and tessoa draws.

```toml
[[pages]]
id = "calculator"
title = "Calculator"
icon = "icon.svg"
single = false
where = ["tab"]
```

| Key | Meaning |
|---|---|
| `id` | Unique among the pages of the plugin, same character rules as the plugin id. This is the `kind` the plugin's `open` receives and `pages-host.open` names. |
| `title` | The menu text, and the title on the page's tab or sidebar section until the page sets its own; `title-zh` and the others for other languages. |
| `icon` | Optional. A `.png` or `.svg` file inside the plugin directory (at most 256 by 256 pixels and 64 KiB), written with `/`, or a name from tessoa's icon set (listed in [pages.md](pages.md#the-tree)). tessoa draws a puzzle piece for a file it cannot use or a name it does not know. |
| `single` | Optional, `false` by default. `true` means one page of this kind per window: opening it again brings the one that is open to the front. Another window may open its own, showing the same data. |
| `where` | Optional, `["tab"]` by default. Where the page opens from the extension page menus of the tab bar and of the file list, from the settings and from `pages-host.open`: `"tab"`, a new tab in the focused pane, or `"sidebar"`, a new section at the bottom of the sidebar. The first place in the list decides; the user can drag an open page between the tabs and the sidebar either way. Any other place is rejected. |

A plugin with pages is built for the world `tessoa-plugin-pages`, which
exports the page functions. Declaring pages in a component built without them
is reported when the plugin is installed.

## `[[columns]]`

One entry per column the plugin adds to tessoa's file list; the user turns
it on in the list's column menu. See [columns.md](columns.md).

```toml
[[columns]]
id = "dims"
title = "Dimensions"
width = 8
align = "end"
sortable = true
when = { ext = ["png", "jpg"], dirs = false }
```

| Key | Meaning |
|---|---|
| `id` | Unique among the columns of the plugin, not empty. This is the column `columns.values` receives. |
| `title` | The header and the name in the column menu; `title-zh` and the others for other languages. |
| `width` | Optional: the width it starts with, in em, from 1 to 100; default 8. |
| `align` | Optional: `start` (the default), `center` or `end`. |
| `sortable` | Optional, `true` by default. |
| `when` | Optional: only for these files. `ext` lists extensions without the dot (empty or absent means any file); `dirs = true` gives folders a value too. |

A column written wrong rejects the manifest, as a command or a page does.

## `[[settings]]`

One entry per option tessoa shows in the plugin's settings page and keeps
for it; the plugin reads the values. See [settings.md](settings.md).

```toml
[[settings]]
id = "format"
title = "Date format"
description = "Used in the names of new files."
type = "choice"
default = "iso"
choices = [{ id = "iso", title = "2026-10-15" }, { id = "us", title = "10/15/2026" }]
```

| Key | Meaning |
|---|---|
| `id` | Unique among the settings of the plugin. |
| `title`, `description` | The name, and an optional dimmed line below it; both take other languages. |
| `type` | `flag`, `choice`, `number` or `text`. |
| `default` | Optional: false, the first choice, `min` (or 0), or empty text without it. |
| `choices` | For a choice: the options, `{ id, title }` each, at least one and at most 256. |
| `min`, `max`, `step`, `unit` | For a number, all optional. |
| `max-length`, `placeholder` | For a text: from 1 to 1024 characters, default 256; a grey hint. |

Unlike a command, a page or a column, a setting written wrong does not
reject the manifest: it is left out, with one problem that lists every
reason.

## Texts in other languages

Every text tessoa shows from the manifest can be given in other languages:
the `title` of a command, a page, a column, a setting and an option of a
choice, and the `description`, `unit` and `placeholder` of a setting. Write
the language after a dash: `title-zh`, `title-ja`, `title-de`. tessoa shows
the one of its interface language, the plain key otherwise.

```toml
[[commands]]
id = "rename-by-date"
title = "Rename by capture date"
title-zh = "按拍摄日期改名"
title-de = "Nach Aufnahmedatum umbenennen"
```

A language tessoa does not show, an empty translation or one that is not
text is ignored and reported. The plugin's `name` has no translations. Texts
the plugin shows itself, on its pages and in its dialogs, come from its code:
`tessoa.ui.language()` (Rust `ui::language()`) says which language to use.

## `[theme]`

Optional: the colours of the plugin's pages. `[theme.light]` is used while
tessoa shows a light theme and `[theme.dark]` while it shows a dark one; a
plugin that gives only one has it used with every theme. Without a theme the
pages show tessoa's colours, and the user can switch a plugin's pages to
tessoa's colours in Settings › System › Extensions.

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
border = "#3d3758"
text-dim = "#8f88ad"
danger = "#ff6b81"
```

| Key | Required | Colours |
|---|---|---|
| `background` | yes | behind the whole page |
| `text` | yes | text |
| `accent` | yes | primary buttons, accent text, the selection, the focus frame |
| `surface` | no | cards, text fields, small menus |
| `border` | no | frames and dividers |
| `text-dim` | no | dim text, placeholders |
| `danger` | no | danger buttons, danger menu items, danger text |

Colours are strings written `"#rrggbb"`, in either case. A missing required
key, or a colour written any other way, makes the whole theme unused: the
manifest still loads and the pages show tessoa's colours. Keys outside the
table are ignored. See [pages.md](pages.md#colours) for what the theme covers
and the contrast it needs; not every colour is used with every component
style.

## `[permissions]`

A table with one array per category. The names are documented in
[permissions.md](permissions.md). Unknown names are rejected.

```toml
[permissions]
files = ["read-selected", "create"]
clipboard = ["write"]
net = ["https://api.example.com", "https://*.example.org/v1/"]
```

`net` lists websites, not permission names: `https://` and a domain name,
optionally with a port and a path the addresses must start with. `*.` in
front of the name stands for any one subdomain; a `*` anywhere else, or
alone, is refused. `http://` is only for this computer: `localhost`,
`127.0.0.1` or `[::1]`, where no port means any port. See
[network.md](network.md).

Write permissions as a table, not as a top-level array: TOML attaches a
top-level key that follows an `[[array]]` entry to that entry, so an array
after `[[commands]]` or `[[pages]]` would become a field of the last entry.
The parser reports that case, for `theme` as well. A table header such as
`[permissions]` or `[theme.dark]` may come anywhere.

## Rules

- Unknown keys are ignored, so a manifest written for a newer version loads
  where it can. A missing `id` or `api` rejects the whole file.
- A few mistakes leave the plugin working: a theme that cannot be used (the
  pages show tessoa's colours), keys of `[theme]` that are ignored, a page
  icon tessoa does not know (it shows a puzzle piece), a setting written
  wrong (it is left out) or keys a setting does not use, a translation into a
  language tessoa does not show. `tessoa-plugin check` reports them and
  fails, every command that builds prints them as warnings, and tessoa's
  plugin log has them.
- A byte order mark at the start of the file is tolerated.
- The manifest is read by `tessoa-plugin check`, by the test host, and by
  tessoa at installation. All three use the same parser.
