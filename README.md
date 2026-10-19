# tessoa extensions

[中文](README.zh-CN.md)

The official extensions of tessoa, the file manager: the plugins tessoa offers
in its extension catalog. Each one is a complete plugin project under
`plugins/`, built against the SDK kit in `sdk/`.

| Directory | Id | Language |
|---|---|---|
| `plugins/calculator` | `com.tessoa.calculator` | TypeScript |
| `plugins/rename-by-date` | `com.tessoa.rename-by-date` | TypeScript |
| `plugins/csv-table` | `com.tessoa.csv-table` | TypeScript |
| `plugins/json-tree` | `com.tessoa.json-tree` | TypeScript |
| `plugins/kanban` | `com.tessoa.kanban` | TypeScript |
| `plugins/hash-files` | `com.tessoa.hash-files` | Rust |
| `plugins/image-size-column` | `com.tessoa.image-size-column` | Rust |

Beside each `plugin.toml` is a `catalog.toml` with the texts of the plugin's
catalog entry: its name and a one-sentence description in English and
Chinese, and the oldest tessoa it needs.

## Setting up

Both languages are used, so run the setup script for each:

```
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts     # Windows
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang rust
bash scripts/setup.sh --lang ts                                      # macOS, Linux
bash scripts/setup.sh --lang rust
```

It installs only what is missing: Node.js, Rust with the `wasm32-wasip2`
target, and the `tessoa-plugin` command-line tool. `scripts/cli.lock` holds
the versions and the sha256 of every download.

## Building

```
scripts/build-all.sh               # every plugin
scripts/build-all.sh kanban        # only the ones named
```

For each plugin it runs `tessoa-plugin test` and then `tessoa-plugin package`,
and stops at the first failure. On Windows, run it from Git Bash. Set
`TESSOA_PLUGIN` to use a `tessoa-plugin` that is not on the PATH.

One plugin can also be worked on by itself, from its directory or with
`--dir plugins/<name>`: `tessoa-plugin test`, `tessoa-plugin package`, and
`tessoa-plugin install` to try it in the tessoa running on this computer.

## Release packages

`scripts/build-all.sh` writes the packages to `dist/<id>-<version>.zip` at the
root and ends with a table of their sha256 and sizes, which is what the
catalog entries need. The version comes from `plugin.toml`: raise it there
for every release of a plugin. `dist/` is build output and is not committed.

## The SDK kit

`sdk/` is the kit the plugin template ships. Do not edit it; replace it with a
newer kit by `tessoa-plugin upgrade` (from the root) and run the build again.
The manual is `sdk/AGENTS.md`, the manifest fields are in
`sdk/docs/guide/manifest.md`.
