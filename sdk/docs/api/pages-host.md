<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# pages-host

What the host does for a plugin's pages.
Available without declaring a permission.

Pages are drawn again after every event and every command without asking, so most plugins only need `open` and `set-title`. Questions go through `ask`: in Rust with `tessoa_plugin_sdk::pages::confirm`, in TypeScript with `await tessoa.ui.confirm(...)` in a page's handler. In Rust these functions need the SDK's `pages` feature.

## Functions

### invalidate

```
wit  invalidate: func(page: page-id)
rust tessoa_plugin_sdk::pages_host::invalidate(page: PageId)
ts   tessoa.pagesHost.invalidate(page: PageId): void
```

Ask for the page to be rendered again. The host calls `render` once
the current call into the plugin returns, at most 20 times a second
per page; several requests in between are one render.

Permission: none.

### open

```
wit  open: func(kind: string, args: option<list<u8>>) -> result<page-id, string>
rust tessoa_plugin_sdk::pages_host::open(kind: &str, args: Option<&[u8]>) -> Result<PageId, String>
ts   tessoa.pagesHost.open(kind: string, args?: unknown): PageId
```

Open a page of this plugin, declared in the manifest under `kind`,
where the first place of its `where` says: a new tab, or a new
section of the sidebar. A page declared `single` that is already
open in the window is focused instead, and its id returned; `args`
then reach no page. From a command, the page opens in the window the
command ran in.

Permission: none.

### set-title

```
wit  set-title: func(page: page-id, title: string)
rust tessoa_plugin_sdk::pages_host::set_title(page: PageId, title: &str)
ts   tessoa.pagesHost.setTitle(page: PageId, title: string): void
```

Change the page's title, shown on its tab or its sidebar section.

Permission: none.

### ask

```
wit  ask: func(page: page-id, title: string, body: string, items: list<string>, ok-label: string, cancel-label: string) -> u64
rust tessoa_plugin_sdk::pages_host::ask(page: PageId, title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> u64
ts   tessoa.pagesHost.ask(page: PageId, title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>
```

Ask a yes/no question on behalf of a page. Returns at once with a
dialog id; the answer arrives as a `dialog` event of that page. With
items, the dialog shows one line per item, like `ui.confirm-list`.
Returns 0 when the page is not open.

Permission: none.

### followed

```
wit  followed: func(page: page-id) -> option<follow-state>
rust tessoa_plugin_sdk::pages_host::followed(page: PageId) -> Option<FollowState>
ts   tessoa.pagesHost.followed(page: PageId): FollowState | undefined
```

The binding and the file of a page that follows the selection; none
when the page does not follow, or tessoa has not said yet. When it
changes, tessoa renders the page again.

Permission: none.

## Types

### file-ref

Defined in [files](files.md#file-ref).

### page-id

`page-id` is `u64` (Rust `u64`, TypeScript `number`).

Identifies one open page. The host issues it when the page opens; it
is unique among all open pages of all plugins.

### follow-source

Variant; Rust `FollowSource`, TypeScript `FollowSource`.

What a page that follows the selection is bound to. A page declared
with `follow = true` shows what the user selects in one tab group;
tessoa draws a chip above the page that names the binding, and the
user changes it there. The page only reads it.

Group and window numbers are the ones on the badges of the tab
groups. They change when the user splits or closes groups, so they
are for showing to the user, not for keeping.

| Case | Payload | Meaning |
|---|---|---|
| `group` | `u32` | A tab group of the page's window, by its number. |
| `window-group` | `tuple<u32, u32>` | A tab group of another window: (window number, group number). |
| `focus` | none | Whichever group has the focus. |
| `tab` | none | One tab, wherever the user moves it. |
| `pinned` | none | The file that was shown when the user pinned it; the page no longer follows anything. |
| `lost` | none | The group or the tab it followed has closed. |

### follow-state

Record; Rust `FollowState`, TypeScript `FollowState`.

The binding of a page that follows the selection, and its file.

| Field | Type | Meaning |
|---|---|---|
| `source` | `follow-source` |  |
| `file` | `option<file-ref>` | The one file selected in the followed group, as a reference issued to this plugin; none while there is no single file: no selection, several files, a folder. The reference may stop working once the file leaves the selection, so a page reads the file when it arrives. |
