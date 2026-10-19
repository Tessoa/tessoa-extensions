# Settings

A plugin lists its options in the manifest; tessoa shows them in its own
settings, checks what the user enters, keeps the values and gives them to the
plugin, which only reads them. The user finds them in Settings › System ›
Extensions: the plugin's row has "Settings…", which opens a page of its
settings with a button that restores the defaults. The plugin can open that
page itself with `tessoa.ui.openSettings()` (Rust `ui::open_settings()`),
from a gear button on its page for instance.

`examples/hash-files-rs` has one: the algorithm, a choice its task reads when
it starts, with a scenario that sets it.

## Declare the settings

```toml
[[settings]]
id = "format"
title = "Date format"
title-zh = "日期格式"
description = "Used in the names of new files."
type = "choice"
default = "iso"
choices = [
  { id = "iso", title = "2026-10-15" },
  { id = "us", title = "10/15/2026" },
]

[[settings]]
id = "size"
title = "Thumbnail size"
type = "number"
default = 64
min = 16
max = 256
step = 16
unit = "px"

[[settings]]
id = "recursive"
title = "Include subfolders"
type = "flag"

[[settings]]
id = "prefix"
title = "Prefix"
type = "text"
max-length = 32
placeholder = "IMG_"
```

| Key | Types | Meaning |
|---|---|---|
| `id` | all | Unique within the plugin; what the plugin reads it by. |
| `title` | all | The setting's name. |
| `description` | all | Optional: a dimmed line below it. |
| `type` | all | `flag` (on or off), `choice` (one of `choices`), `number` or `text`. |
| `default` | all | What the plugin reads until the user changes it. Without it: off, the first choice, `min` (or 0 within the range), or empty text. A number's default lies in its range and on its step. |
| `choices` | choice | The options, each `{ id, title }`; at least one, at most 256, ids of at most 128 bytes. |
| `min`, `max`, `step`, `unit` | number | All optional. The value stays in the range and on the step, counted from `min` (or 0); `unit` is shown after it, such as `px`. |
| `max-length`, `placeholder` | text | The longest text, from 1 to 1024 characters, default 256; a grey hint while it is empty. |

`title`, `description`, `unit`, `placeholder` and the `title` of an option
take translations: `title-zh`, `title-ja`, `title-de` and so on. tessoa shows
the one of its interface language, the plain key otherwise.

A setting written wrong is left out, and one problem lists every reason; a
key a setting does not use is ignored and reported. `tessoa-plugin check`
reports both, every command that builds prints them, and so does the plugin
log.

## Read them

TypeScript: `tessoa.settings.get(id)` returns the value as its type has it:
a `flag` a boolean, a `number` a number, a `choice` the id of the chosen
option, a `text` text.

```tsx
render: (s) => {
  const format = tessoa.settings.get("format");
  return <text key="today">{formatDay(s.today, format === "us")}</text>;
},
```

Rust: one function per type.

```rust
let format = settings::text("format");       // a choice reads as the option's id
let size = settings::number("size");
let recursive = settings::flag("recursive");
```

`settings.kind(id)` says the type of a declared setting. Reading costs
nothing, wherever the plugin runs: in a page, a command, a task or a column.
An id the manifest does not declare, or a setting read as another type, reads
as `false`, 0 or empty text (TypeScript `get`: `undefined`), and the plugin
log gets one line about it.

**Read a setting where it is used**, in a page's `render` for instance: when
the user changes settings, every open page of the plugin renders again, so a
page that reads them while it renders shows the new values without more code.
Rust pages also hear `Event::SettingsChanged { ids }`, and a plugin without
pages `PluginEvent::SettingsChanged` in `Plugin::on_event`. The plugin's
column values are all asked for again.

The plugin cannot change its settings: they are the user's. Data the plugin
keeps for itself belongs in its storage.

## Testing

`[settings]` gives the values the user set before the scenario, and
`set-setting` changes one in a step, as the user would in tessoa; both are
held to the setting's rules. See [testing.md](testing.md).

```toml
[settings]
format = "us"

[[steps]]
open-page = "today"
expect-text = { key = "today", text = "10/15/2026" }

[[steps]]
set-setting = { id = "format", value = "iso" }
expect-text = { key = "today", text = "2026-10-15" }
```
