# The command-line tool

`tessoa-plugin` works on a plugin project: a directory with a `plugin.toml`.
Run it inside the project, or point it there with `--dir`.

| Command | What it does | Exit status |
|---|---|---|
| `init` | Run once in a project made from the template. `--lang ts` or `--lang rust` and `--id <reverse.domain.id>` (and optionally `--name`) move the chosen skeleton from `skeleton/` to the project root with the id and name filled in, and delete the other skeleton. The template's own `README*.md` files at the root are replaced by the skeleton's README, which describes the plugin and goes into its package. Refuses a project that already has a `plugin.toml`, and a name with quotes or backslashes. | 1 on failure |
| `upgrade` | Updates the tool itself when a newer release for this platform exists, then replaces `sdk/` with the newest SDK kit: the TypeScript declarations, the Rust SDK, the interface definition, the generated API reference, the guides and the manual `sdk/AGENTS.md`. Both come from a manifest published under `download.tessoa.com`; its signature and each archive's sha256 are checked before anything on disk changes, the old `sdk/` stays if anything fails, and a kit that needs a newer tool than the one running is refused. On Windows the replaced executable is kept as `tessoa-plugin.exe.old` until the next run. `--check` only reports; `--version` picks a kit. Everything outside `sdk/` is left alone. No account is needed. | 1 on failure, also when only the tool could not be updated |
| `doctor` | Checks that the toolchain the project needs is installed: cargo, rustc and the `wasm32-wasip2` target for a Rust project; Node.js and npm for a TypeScript project. Reports whether the manifest parses and whether the plugin is built. Fails while the template's `skeleton/` is still there: before `init`, it says to run `init`; beside a `plugin.toml` written by hand, which `init` refuses, it says to delete the folder. | 1 when something required is missing or `skeleton/` is left |
| `check` | Prints the manifest summary and the import audit of the built component, then the problems of the manifest, one `problem:` line each: a theme that cannot be used, keys of `[theme]` that are ignored, a page icon tessoa does not know, theme colours with too little contrast or a palette under the wrong name ([pages.md](pages.md#colours)). | 1 when the audit finds a violation or there is a problem |
| `build` | Builds with the project's own toolchain (`cargo build --release --target wasm32-wasip2`, or `npm run build` after an `npm install` when `node_modules` is missing), copies the result to the entry path named in the manifest, then audits it. The problems `check` reports are printed as `warning:` lines, here and in every command that builds. | 1 when the build fails or the audit finds a violation |
| `test` | Runs the scenarios in `tests/scenarios` against the plugin, with fixtures from `tests/fixtures`: scenarios that run a command, and page scenarios with steps. `--scenarios` and `--fixtures` change the directories; positional names filter by file name; `--no-build` skips the build. A failed page scenario shows the open pages as `render` prints them. | 1 when a scenario fails |
| `render` | Prints a page as text, one line per node: its kind, what it shows, its key and the attributes that are not the default ([pages.md](pages.md#testing-a-page)). `--page <id>` picks the kind (default: the only page the manifest declares); `--steps <scenario>` runs the steps of a page scenario first and prints the pages they left open, or the page of `--page`; a list, a table or a tree shows the rows on its screen, or `--rows 100..140`; `--fixtures` and `--no-build` as for `test`. | 1 when the page cannot be shown or a step fails |
| `run` | Runs one command on files you name: `--command rename-by-date --file a.jpg --file b.jpg`. Dialogs are answered by `--answer ok` / `--answer cancel` in order, or from the terminal when no answer is given. Prints the log, the dialogs, the renames and the notifications as they happen. The files are really renamed. The tasks the command starts run to their end, at most ten minutes; requests go to the websites. | 1 when the command fails |
| `package` | Writes `dist/<id>-<version>.zip` (or `--out`): the manifest, the entry file, the page icons the manifest names, the `images` folder (the pictures of `image` nodes), and README, LICENSE and icon.png when present. Refuses a plugin that fails the audit. | 1 on failure |
| `install` | Builds (unless `--no-build`), audits, then hands the project folder to tessoa by running `tessoa --dev-extension=<dir>`. tessoa loads the plugin from that folder, asks you to confirm its permissions in the tessoa window the first time, and reloads it after every later build. tessoa is found through `TESSOA_EXE`, the PATH, the installer's locations and, on Windows, the App Paths registry key. A running tessoa takes the folder over its own channel (a named pipe on Windows, a Unix socket on macOS and Linux); when none is running, the tessoa started here becomes the running one. A tessoa from before the plugin platform is recognised and refused instead of being started. | 1 when the build or audit fails or tessoa refuses; 2 when tessoa cannot be found, is too old, or the folder is not a plugin project |

There is no option that grants a permission. A plugin has what its manifest
declares; the user grants anything beyond that inside tessoa.

## Where things go

- The build output lands at the entry path from the manifest, usually
  `dist/plugin.wasm` or `dist/plugin.js`. `dist/` is build output and is not
  committed.
- Compiled components are cached in the system's temporary directory under
  `tessoa-plugin/cache`. Deleting it costs one recompilation.
- The template's setup scripts install the tool, and Node.js when the system
  has none recent enough, under `~/.tessoa-plugin` (or `TESSOA_PLUGIN_HOME`),
  and Rust with rustup under `~/.cargo` (or `CARGO_HOME`). The tool adds those
  folders to the PATH of the programs it starts, so it finds what the scripts
  installed even in a terminal that was opened before them.
