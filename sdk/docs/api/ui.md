<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# ui

Notifications and dialogs. The host draws them with its own widgets, so they
follow the theme, the UI scale and the language.
Available without declaring a permission.

## Functions

### notify

```
wit  notify: func(title: string, body: string)
rust tessoa_plugin_sdk::ui::notify(title: &str, body: &str)
ts   tessoa.ui.notify(title: string, body: string): void
```

Show a notification and return immediately.

Permission: none.

### confirm

```
wit  confirm: func(title: string, body: string, ok-label: string, cancel-label: string) -> bool
rust tessoa_plugin_sdk::ui::confirm_with(title: &str, body: &str, ok_label: &str, cancel_label: &str) -> bool   (ui::confirm(title, body) has the labels OK and Cancel)
ts   tessoa.ui.confirm(title: string, body: string, okLabel?: string, cancelLabel?: string): Promise<boolean>
```

Ask a yes/no question and wait for the answer.
Returns true when the user pressed the `ok-label` button.
A page must not wait: in a page call this returns false at once and
the call is reported as refused. Pages ask with `pages-host.ask`,
which `tessoa.ui.confirm` in TypeScript does by itself in a page.

Permission: none.

### confirm-list

```
wit  confirm-list: func(title: string, body: string, items: list<string>, ok-label: string, cancel-label: string) -> bool
rust tessoa_plugin_sdk::ui::confirm_list_with(title: &str, body: &str, items: &[String], ok_label: &str, cancel_label: &str) -> bool   (ui::confirm_list(title, body, items) has the labels OK and Cancel)
ts   tessoa.ui.confirmList(title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>
```

Like `confirm`, with one line per item shown between the body and the
buttons. Use it to preview a batch change before applying it.

Permission: none.

### language

```
wit  language: func() -> string
rust tessoa_plugin_sdk::ui::language() -> String
ts   tessoa.ui.language(): string
```

The language tessoa shows its interface in, as a short code such as
`en`, `zh`, `ja` or `de`. Use it to pick the plugin's own texts.

Permission: none.

### open-url

```
wit  open-url: func(url: string) -> result<_, string>
rust tessoa_plugin_sdk::ui::open_url(url: &str) -> Result<(), String>
ts   tessoa.ui.openUrl(url: string): void
```

Offer to open a web address in the user's browser. tessoa shows the
whole address and opens it only when the user agrees; the address
need not be declared anywhere. Only `http` and `https` addresses are
accepted, and only while the plugin handles something the user just
did: a command, or a click, a key or a menu choice on a page. At
other times, and while the plugin's previous offer is still on
screen, the call fails with the reason.

Permission: none.

### open-settings

```
wit  open-settings: func()
rust tessoa_plugin_sdk::ui::open_settings()
ts   tessoa.ui.openSettings(): void
```

Open the plugin's page in tessoa's settings, where the settings the
manifest declares are. Does nothing for a plugin without settings.

Permission: none.
