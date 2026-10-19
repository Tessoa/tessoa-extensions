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

## Types

### page-id

`page-id` is `u64` (Rust `u64`, TypeScript `number`).

Identifies one open page. The host issues it when the page opens; it
is unique among all open pages of all plugins.
