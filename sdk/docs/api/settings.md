<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# settings

The plugin's settings: the options `[[settings]]` in `plugin.toml`
declares. tessoa shows them in its settings, checks and keeps the values;
the plugin only reads them, without a round trip. A setting the user never
changed has the default the manifest gives. When the user changes
settings, `plugin.on-event` hears of it.
Available without declaring a permission.

## Functions

### kind

```
wit  kind: func(id: string) -> option<string>
rust tessoa_plugin_sdk::settings::kind(id: &str) -> Option<String>
ts   tessoa.settings.kind(id: string): string | undefined
```

The type of a declared setting: `flag`, `choice`, `number` or `text`;
none for an id the manifest does not declare.

Permission: none.

### flag

```
wit  flag: func(id: string) -> bool
rust tessoa_plugin_sdk::settings::flag(id: &str) -> bool
ts   tessoa.settings.flag(id: string): boolean
```

The value of a `flag` setting. False for any other id; an id the
manifest does not declare also writes a line to the plugin log.

Permission: none.

### number

```
wit  number: func(id: string) -> f64
rust tessoa_plugin_sdk::settings::number(id: &str) -> f64
ts   tessoa.settings.number(id: string): number
```

The value of a `number` setting, within its minimum and maximum and
on its step. 0 for any other id, as with `flag`.

Permission: none.

### text

```
wit  text: func(id: string) -> string
rust tessoa_plugin_sdk::settings::text(id: &str) -> String
ts   tessoa.settings.text(id: string): string
```

The value of a `text` setting, or the id of the chosen option of a
`choice` setting. Empty for any other id, as with `flag`.

Permission: none.

TypeScript also has:

```
tessoa.settings.get(id: string): boolean | number | string | undefined
```
