<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# storage

The plugin's own storage: keys and values that survive restarts, are
shared by all of the plugin's pages and commands, and go with tessoa's
settings backup. At most 1 MiB in total, keys and values counted; one
value at most 256 KiB, one key at most 128 bytes.
Available without declaring a permission.

## Functions

### get

```
wit  get: func(key: string) -> option<list<u8>>
rust tessoa_plugin_sdk::storage::get(key: &str) -> Option<Vec<u8>>
ts   tessoa.storage.get(key: string): Uint8Array | undefined
```

The value stored under `key`, if any.

Permission: none.

### set

```
wit  set: func(key: string, value: list<u8>) -> result<_, error>
rust tessoa_plugin_sdk::storage::set(key: &str, value: &[u8]) -> Result<(), Error>
ts   tessoa.storage.set(key: string, value: Uint8Array): void
```

Store `value` under `key`, replacing what was there.

Permission: none.

### remove

```
wit  remove: func(key: string)
rust tessoa_plugin_sdk::storage::remove(key: &str)
ts   tessoa.storage.remove(key: string): void
```

Remove `key`. Removing a key that does not exist does nothing.

Permission: none.

### keys

```
wit  keys: func() -> list<string>
rust tessoa_plugin_sdk::storage::keys() -> Vec<String>
ts   tessoa.storage.keys(): string[]
```

Every key, in byte order.

Permission: none.

## Types

### error

Enum; Rust `Error`, TypeScript `StorageError` (thrown) with `kind: ErrorKind`.

| Case | Meaning |
|---|---|
| `quota` | The value, or the storage as a whole, would exceed its limit. |
| `key-too-long` | The key is longer than 128 bytes. |
