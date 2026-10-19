<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# clipboard

The system clipboard.

## Functions

### write-text

```
wit  write-text: func(text: string) -> result<_, error>
rust tessoa_plugin_sdk::clipboard::write_text(text: &str) -> Result<(), Error>
ts   tessoa.clipboard.writeText(text: string): void
```

Put text on the clipboard.
Requires the `clipboard.write` permission.

Permission: `clipboard.write`.

## Types

### error

Defined in [files](files.md#error).
