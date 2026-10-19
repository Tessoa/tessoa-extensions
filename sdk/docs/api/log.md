<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# log

Plugin log. Lines go to the plugin log panel and the plugin log file.
Available without declaring a permission.

## Functions

### log

```
wit  log: func(level: level, message: string)
rust tessoa_plugin_sdk::log::log(level: Level, message: &str)
ts   tessoa.log.log(level: Level, message: string): void
```

Write one line to the plugin log.

Permission: none.

TypeScript also has:

```
tessoa.log.debug(message: string): void
tessoa.log.info(message: string): void
tessoa.log.warn(message: string): void
tessoa.log.error(message: string): void
```

## Types

### level

Enum; Rust `Level`, TypeScript `Level`.

| Case | Meaning |
|---|---|
| `debug` |  |
| `info` |  |
| `warn` |  |
| `error` |  |
