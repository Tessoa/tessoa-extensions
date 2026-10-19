<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# images

Images the plugin makes, for `image` nodes and image cells. The bytes go
to tessoa once, when the image is added; pages refer to it by number. The
numbers are valid until the plugin stops: a plugin that starts again adds
its images again, so a page's state should not keep them.
Available without declaring a permission.

## Functions

### add

```
wit  add: func(data: list<u8>) -> result<u64, error>
rust tessoa_plugin_sdk::images::add(data: &[u8]) -> Result<u64, Error>
ts   tessoa.images.add(data: Uint8Array | string): number
```

Add an image from the bytes of a PNG, JPEG, GIF (its first frame),
WebP, BMP or SVG file: at most 8 MiB, and at most 4096 by 4096 pixels.
Returns its number.

Permission: none.

### add-rgba

```
wit  add-rgba: func(width: u32, height: u32, data: list<u8>) -> result<u64, error>
rust tessoa_plugin_sdk::images::add_rgba(width: u32, height: u32, data: &[u8]) -> Result<u64, Error>
ts   tessoa.images.addRgba(width: number, height: number, data: Uint8Array): number
```

Add an image from pixels: `width` times `height` times 4 bytes of
red, green, blue and alpha, row by row from the top. At most 4096 by
4096 pixels. Returns its number.

Permission: none.

### remove

```
wit  remove: func(id: u64)
rust tessoa_plugin_sdk::images::remove(id: u64)
ts   tessoa.images.remove(id: number): void
```

Remove an image. Nodes and cells that still name it show an empty
frame.

Permission: none.

## Types

### error

Variant; Rust `Error`, TypeScript `ImagesError` (thrown) with `kind: ErrorKind`.

| Case | Payload | Meaning |
|---|---|---|
| `quota` | none | The plugin's images would take more than 64 MiB once decoded; remove some first. |
| `invalid` | `string` | The data is not an image tessoa reads, or it is too large. Carries the reason. |
