# The command-line tool

`tessoa-plugin` works on a plugin project: a directory with a `plugin.toml`.
Run it inside the project, or point it there with `--dir`. With `--json`
it writes what it found as one JSON document for a program to read
([below](#machine-readable-output)).

| Command | What it does | Exit status |
|---|---|---|
| `new` | Creates a project without the template: `new <dir> --lang ts\|rust --id <reverse.domain.id>` (and optionally `--name`) writes what `init` leaves in a copy of the template, without its examples, setup scripts and licence: the skeleton of the language with the id and name filled in, `AGENTS.md` and the one-line files that point other tools to it, `.gitignore`, `.gitattributes`, the SDK kit in `sdk/`, and the skill for coding tools the template carries, as `skill install --project` writes it ([ai-tools.md](ai-tools.md)). The skeleton is built into the tool. The kit is the one released with the tool, or else the newest older one, downloaded and checked the way `upgrade` does; `--kit` takes a kit zip or an unpacked kit folder instead. Refuses a folder that exists and is not empty, and an id or name `init` would refuse. Does not run git, npm or cargo. | 1 when the kit cannot be downloaded or written; 2 when the folder exists and is not empty, the id or name cannot be used, or `--kit` is not a kit |
| `init` | Run once in a project made from the template. `--lang ts` or `--lang rust` and `--id <reverse.domain.id>` (and optionally `--name`) move the chosen skeleton from `skeleton/` to the project root with the id and name filled in, and delete the other skeleton. The template's own `README*.md` files at the root are replaced by the skeleton's README, which describes the plugin and goes into its package. Refuses a project that already has a `plugin.toml`, and a name with quotes or backslashes. | 1 on failure; 2 when the folder has no `skeleton/` or already has a `plugin.toml`, or the id or name cannot be used |
| `upgrade` | Updates the tool itself when a newer release for this platform exists, then replaces `sdk/` with the newest SDK kit: the TypeScript declarations, the Rust SDK, the interface definition, the generated API reference, the guides, the manual `sdk/AGENTS.md` and the changelog `sdk/CHANGELOG.md`, whose sections between the old kit and the new one it prints afterwards. Both come from a manifest published under `download.tessoa.com`; its signature and each archive's sha256 are checked before anything on disk changes, the old `sdk/` stays if anything fails, and a kit that needs a newer tool than the one running is refused. On Windows the replaced executable is kept as `tessoa-plugin.exe.old` until the next run. `--check` only reports; `--version` picks a kit. Everything outside `sdk/` is left alone. No account is needed. | 1 on failure, also when only the tool could not be updated; 2 outside a project |
| `doctor` | Checks that the toolchain the project needs is installed: cargo, rustc and the `wasm32-wasip2` target for a Rust project; Node.js and npm for a TypeScript project. Reports whether the manifest parses and whether the plugin is built. Fails while the template's `skeleton/` is still there: before `init`, it says to run `init`; beside a `plugin.toml` written by hand, which `init` refuses, it says to delete the folder. | 2 when something required is missing, there is no project or `skeleton/` is left; 1 when `plugin.toml` cannot be read |
| `check` | Prints the manifest summary and the import audit of the built component, then the problems of the manifest, one `problem:` line each: a theme that cannot be used, keys of `[theme]` that are ignored, a page icon tessoa does not know, theme colours with too little contrast or a palette under the wrong name ([pages.md](pages.md#colours)). | 1 when the plugin is not built, the audit finds a violation or there is a problem; 2 outside a project |
| `build` | Builds with the project's own toolchain (`cargo build --release --target wasm32-wasip2`, or `npm run build` after an `npm install` when `node_modules` is missing), copies the result to the entry path named in the manifest, then audits it. The problems `check` reports are printed as `warning:` lines, here and in every command that builds. | 1 when the build fails or the audit finds a violation; 2 outside a project, or when cargo or npm cannot be run |
| `test` | Runs the scenarios in `tests/scenarios` against the plugin, with fixtures from `tests/fixtures`: scenarios that run a command, and page scenarios with steps. `--scenarios` and `--fixtures` change the directories; positional names filter by file name; `--no-build` skips the build. A failed page scenario shows the open pages as `render` prints them. | 1 when a scenario fails; 2 outside a project or when no scenario is found |
| `render` | Prints a page as text, one line per node: its kind, what it shows, its key and the attributes that are not the default ([pages.md](pages.md#testing-a-page)). `--page <id>` picks the kind (default: the only page the manifest declares); `--steps <scenario>` runs the steps of a page scenario first and prints the pages they left open, or the page of `--page`; a list, a table or a tree shows the rows on its screen, or `--rows 100..140`; `--fixtures` and `--no-build` as for `test`. | 1 when the page cannot be shown or a step fails; 2 outside a project, when the manifest declares no page or the `--steps` file cannot be read |
| `run` | Runs one command on files you name: `--command rename-by-date --file a.jpg --file b.jpg`. Dialogs are answered by `--answer ok` / `--answer cancel` in order, or from the terminal when no answer is given. Prints the log, the dialogs, the renames and the notifications as they happen. The files are really renamed. The tasks the command starts run to their end, at most ten minutes; requests go to the websites. | 1 when the command fails; 2 outside a project, for a command the manifest does not declare, or a file that does not exist |
| `package` | Writes `dist/<id>-<version>.zip` (or `--out`): the manifest, the entry file, the page icons the manifest names, the `images` folder (the pictures of `image` nodes), and README, LICENSE and icon.png when present. Refuses a plugin that fails the audit. | 1 on failure; 2 outside a project |
| `install` | Builds (unless `--no-build`), audits, then hands the project folder to tessoa by running `tessoa --dev-extension=<dir>`. tessoa loads the plugin from that folder, asks you to confirm its permissions in the tessoa window the first time, and reloads it after every later build. tessoa is found through `TESSOA_EXE`, the PATH, the installer's locations and, on Windows, the App Paths registry key; when `TESSOA_EXE` is set, it is the only place looked at, so a wrong path is reported instead of falling back to another tessoa. A running tessoa takes the folder over its own channel (a named pipe on Windows, a Unix socket on macOS and Linux); when none is running, the tessoa started here becomes the running one. A tessoa from before the plugin platform is recognised and refused instead of being started. | 1 when the build or audit fails or tessoa refuses; 2 outside a project, or when tessoa cannot be found, is too old, or says the folder is not a plugin project |
| `dev` | Builds and audits, hands the project to tessoa as `install` does once a build passes (not with `--no-install`), then watches `src/`, `images/`, `plugin.toml`, `package.json` and `Cargo.toml` and builds and audits again after every change, printing one line with the time and the outcome; `--test` runs the scenarios after every build that passes. tessoa reloads the plugin after each build by itself. A failed build is reported and the next change tries again. The files are looked at twice a second and a burst of changes starts one build. Runs until Ctrl+C. | 0 when stopped with Ctrl+C; 2 outside a project, or with `--json`, since it never ends and has no report to write |
| `skill install` | Installs the skill that teaches a coding tool to write tessoa plugins, for authors who did not start from the template (a project made from it already has the skill). It goes into the user's own skills, which every project sees, or with `--project` into the project at hand (`--dir`, or the nearest folder with a `plugin.toml` or an `sdk/VERSION`). A skill that is already current is left alone; one that differs is replaced. `--check` only reports. Where the file goes and what it holds: [ai-tools.md](ai-tools.md). | 1 on failure; with `--check`, 1 when the skill is missing or differs; 2 when the user folder cannot be found |
| `mcp` | Serves the commands to a coding tool as an MCP server: JSON-RPC 2.0 on stdin and stdout, one message per line. The tools are `new_project`, `doctor`, `check`, `build`, `test`, `render`, `package` and `install`, which run the commands of the same names and return their `--json` document, and `api_reference`, `manifest_reference`, `guide`, `list_examples` and `example_source`, which serve the API reference, the manifest keys, the manual and the guides, and the examples built into the tool; the same documents are offered as resources. Every command tool takes `dir`, the absolute path of the project. What the commands print for a person goes to stderr. Started by the client, with no arguments; how to set it up: [ai-tools.md](ai-tools.md#mcp-server). `--json` changes nothing here. | 0 when the client closes stdin; 2 when stdin is a terminal or its first line is not a JSON-RPC message |

Every command exits with one of three statuses, and none other:

- **0**: it did what it was asked.
- **1**: it ran, and the plugin, its build, its tests, the kit download or
  tessoa's answer failed.
- **2**: it could not start: the arguments are wrong, the folder is not a
  plugin project (or not one set up with `init`), or a tool it needs is
  missing or too old. Fixing the setup comes before fixing the plugin.

There is no option that grants a permission. A plugin has what its manifest
declares; the user grants anything beyond that inside tessoa.

Every problem the tool reports comes with a hint on how to fix it, printed on
the next line:

```
problem: theme.light: text #aaaaaa on background #ffffff has a contrast of 2.3:1; text needs at least 4.5:1
  hint: Pick colours with more contrast; sdk/docs/guide/pages.md#colours gives the limits.
```

## Machine-readable output

`--json` works with every command. stdout then carries nothing but one JSON
document, written when the command ends; the lines meant for a person,
npm's output included, go to stderr, so a build still shows its progress.
The exit status is the same as without the flag. `run --json` never asks the
terminal: dialogs are answered from `--answer`, and the ones past the last
answer count as cancelled. A mistake in the arguments is reported as JSON
too, with the code `usage.invalid`; `--help` and `--version` print as usual.
The MCP server (`mcp`) returns the same document as the structured result of
each tool that runs a command.

The document:

| Field | |
|---|---|
| `schema` | The version of this shape, now `1`. Fields may be added; none is renamed or removed without raising it. |
| `command` | The command that ran, such as `check`. |
| `ok` | Whether it did what it was asked: `exit` is 0. |
| `exit` | The exit status, 0, 1 or 2. |
| `result` | What the command found (below), or `null` when it stopped before it had anything. |
| `problems` | What went wrong or needs a look, in order. Each has a `code` (the table below), a `message`, a `hint`, and when it is about one file, its `path` and sometimes its `line`. In `build` and the commands that build, the problems of the manifest are listed here without failing the command; `check` fails on them. |

The `result` of each command:

| Command | `result` |
|---|---|
| `doctor` | `tool_version`; `project` (`dir`, `id`, `language`: `rust` or `typescript`) when there is one; `checks`, each with a `name` (`project`, `skeleton`, `cargo`, `rustc`, `wasm32-wasip2`, `node`, `npm`, `entry`), a `status` (`ok`, `fail`, or `info` for what is not required) and a `detail`. |
| `check` | `manifest`: `id`, `version`, `name`, `language`, `api`, `commands` and `pages` (`id`, `title`, `placements`, and `single` for a page), `permissions`, `theme` (`none`, `light-and-dark`, `light-only`, `dark-only`). `audit`: `passed`, `imports` (`name`, `functions`), `required_permissions`, `violations`. |
| `build` | `entry`, the built file, and `audit` as for `check`. |
| `test` | `passed` and `failed`, the counts, and `scenarios`, each with `name`, `file` (relative to the scenarios directory), `passed`, `failures`, and when they apply `result` (how the command ended: `ok` or the runner's code), `load_error`, and `pages`, the lines `render` would print, for a failed page scenario. |
| `render` | `lines`: the pages, one line per node, as printed. When a step fails, the lines of the pages it left. |
| `run` | `command`; `end`, `ok` or the runner's code (`trap`, `cpu-burst`, `cpu-budget`, `memory-limit`, `plugin-error`, ...) and its `message`; `events`, what tessoa would have shown, in the order it happened, each with a `type`: `log` (`level`, `message`), `dialog` (`kind`, `title`, `body`, `items`, `answer`: `ok`, `cancel` or `unanswered`), `notification` (`title`, `body`), `rename` (`from`, `to`, `outcome`), `refused` (`call`, `reason`), `transfer` (`kind`, `files`), `clipboard` (`text`), `page-failed` (`page`, `code`, `message`), `open-url` (`url`), `task-started` (`task`, `title`), `task-done` (`task`, `end`); `unfinished` when tasks still ran at the time limit; `host_calls`, `busy_ms`, `wall_ms`. A task's events come in when the runner next looks at it, so they can follow a line printed later. |
| `package` | `path`, `size`, `sha256`, and `files`, the archive's entries. |
| `install` | `tessoa`, the executable used; `channel`: `running` when the running tessoa took the folder, `started` when the tessoa started here became it; `outcome`: `accepted`, `started`, `refused`, `not-found`, `too-old` or `audit-failed`; `refused_status`, tessoa's exit status when it refused. |
| `init` | `dir`, `id`, `name`, `lang` (`ts` or `rust`), and `kit`, the version in `sdk/`. |
| `upgrade` | `check_only`; `kit_before`, `kit_after` and `kit_available`, with the `api` of the available kit; `tool_before`, `tool_after`, and `newer_tool` when the manifest has one; `up_to_date`. |

A complete example, `doctor --json` in a template that was not set up with
`init`:

```json
{
  "schema": 1,
  "command": "doctor",
  "ok": false,
  "exit": 2,
  "result": {
    "tool_version": "0.2.0",
    "checks": [
      { "name": "project", "status": "fail", "detail": "project: /home/ana/my-plugin has no plugin.toml" },
      { "name": "skeleton", "status": "fail", "detail": "skeleton/ is still here: run tessoa-plugin init --lang ts|rust --id <reverse.domain.id> --name \"<Name>\"" },
      { "name": "cargo", "status": "ok", "detail": "cargo 1.92.0 (344c4567c 2025-10-21)" },
      { "name": "rustc", "status": "ok", "detail": "rustc 1.92.0 (ded5c06cf 2025-12-08)" },
      { "name": "wasm32-wasip2", "status": "ok", "detail": "target wasm32-wasip2 installed" },
      { "name": "node", "status": "ok", "detail": "node v22.14.0" },
      { "name": "npm", "status": "ok", "detail": "npm 11.8.0" }
    ]
  },
  "problems": [
    {
      "code": "project.not-found",
      "message": "project: /home/ana/my-plugin has no plugin.toml",
      "hint": "Run inside a plugin project or pass --dir <project>; a project fresh from the template needs tessoa-plugin init first."
    },
    {
      "code": "project.skeleton-left",
      "message": "skeleton/ is still here: run tessoa-plugin init --lang ts|rust --id <reverse.domain.id> --name \"<Name>\"",
      "hint": "Before init: run tessoa-plugin init --lang ts|rust --id <reverse.domain.id>. With a plugin.toml written by hand: delete skeleton/."
    }
  ]
}
```

## Problem codes

A code names a kind of problem and does not change; the message says what
happened this time. The first part of a code is its area. The exit column
is the status when a problem of that kind stops the command.

| Code | When | Exit | Hint |
|---|---|---|---|
| `usage.invalid` | The arguments are not valid. | 2 | Run tessoa-plugin <command> --help for the options; sdk/docs/guide/cli.md lists every command. |
| `usage.unknown-command` | `run --command` names a command the manifest does not declare. | 2 | Pass one of the command ids that plugin.toml declares under [[commands]]. |
| `usage.file-not-found` | A file given to `run --file` or `render --steps` does not exist. | 2 | Pass files that exist; paths are relative to the current directory. |
| `usage.no-pages` | `render` on a plugin without pages. | 2 | render shows pages; declare one under [[pages]] (sdk/docs/guide/manifest.md#pages), or use test or run for commands. |
| `usage.bad-id` | `init --id` or `new --id` is not a plugin id. | 2 | Use a reverse-domain id in lower case with at least two parts, such as com.example.my-plugin. |
| `usage.bad-name` | `init --name` or `new --name` cannot be used. | 2 | Leave quotes, backslashes and control characters out of --name. |
| `project.not-found` | No `plugin.toml` (for `upgrade`, no `sdk/VERSION` either) in the folder or above it. | 2 | Run inside a plugin project or pass --dir <project>; a project fresh from the template needs tessoa-plugin init first. |
| `project.skeleton-left` | `skeleton/` is still in the project. | 2 | Before init: run tessoa-plugin init --lang ts\|rust --id <reverse.domain.id>. With a plugin.toml written by hand: delete skeleton/. |
| `project.not-template` | `init` in a folder without `skeleton/`. | 2 | init runs once, in a project made from the template; this one has no skeleton/ folder. |
| `project.initialised` | `init` in a project that has a `plugin.toml`. | 2 | The project already has a plugin.toml; init is not needed. Run tessoa-plugin doctor. |
| `new.target-exists` | `new` is given a folder that exists and is not empty, or a file. | 2 | Pass a folder that does not exist yet, or an empty one; new never writes over files. |
| `new.kit-invalid` | `new --kit` names something that is not a kit zip or an unpacked kit. | 2 | Pass --kit a kit zip or an unpacked kit folder with a VERSION file, or leave it out to download the kit. |
| `new.kit-download` | `new` cannot download the kit. A kit that fails its checks keeps the check's code: `kit.signature`, `kit.hash`, `kit.schema`. | 1 | Check the network connection and any proxy, or pass --kit with a kit zip or an unpacked kit folder to work offline. |
| `dev.no-json` | `dev` with `--json`. | 2 | dev runs until Ctrl+C and has no report to write; run it without --json, or run build, test and install one at a time. |
| `skill.no-home` | `skill install` cannot find the user folder. | 2 | Set HOME (USERPROFILE on Windows) to your user folder, or install into the project with --project. |
| `skill.write` | `skill install` cannot write the skill file. | 1 | Check that the folder named in the message can be written, then run tessoa-plugin skill install again. |
| `skill.not-current` | `skill install --check` finds the skill missing or different. | 1 | Run tessoa-plugin skill install (with --project for the project's copy). |
| `mcp.not-protocol` | `mcp` is started in a terminal, or its first line is not a JSON-RPC message. | 2 | tessoa-plugin mcp is started by an editor or another MCP client, which writes JSON-RPC to its stdin; see sdk/docs/guide/ai-tools.md#mcp-server. |
| `mcp.bad-arguments` | An MCP tool is called with arguments its schema does not allow, or `dir` is not an absolute path. | 2 | Pass the arguments the tool's inputSchema in tools/list describes; paths must be absolute. |
| `mcp.unknown-name` | `api_reference`, `manifest_reference`, `guide` or `example_source` is asked for a name it does not know; the message lists the names it does. | 2 | Call the tool without that argument to list the names it knows. |
| `env.cargo-missing` | cargo cannot be run. | 2 | Install Rust: run the template's setup script with --lang rust, or get it from https://rustup.rs. |
| `env.rustc-missing` | rustc cannot be run. | 2 | Install Rust: run the template's setup script with --lang rust, or get it from https://rustup.rs. |
| `env.wasm-target-missing` | The `wasm32-wasip2` target is not installed. | 2 | Run: rustup target add wasm32-wasip2 |
| `env.node-missing` | Node.js cannot be run. | 2 | Install Node.js 22 or newer: run the template's setup script with --lang ts. |
| `env.npm-missing` | npm cannot be run. | 2 | npm comes with Node.js; run the template's setup script with --lang ts. |
| `env.tessoa-not-found` | `install` cannot find tessoa, or `TESSOA_EXE` names a file that does not exist. | 2 | Install tessoa, or set TESSOA_EXE to the path of its executable. Do not work around it. |
| `env.tessoa-too-old` | The tessoa found is older than the plugin platform. | 2 | Update tessoa to a version with the plugin platform, or set TESSOA_EXE to a newer one. |
| `env.engine` | The WebAssembly engine does not start. | 1 | The WebAssembly engine could not start; check that the system's temporary directory is writable. |
| `manifest.invalid` | `plugin.toml` cannot be read; the plugin cannot load. | 1 | Fix plugin.toml as the message says; sdk/docs/guide/manifest.md describes every key. |
| `manifest.theme-contrast` | Theme colours with too little contrast. | 1 | Pick colours with more contrast; sdk/docs/guide/pages.md#colours gives the limits. |
| `manifest.theme-side` | A dark palette under `[theme.light]`, or a light one under `[theme.dark]`. | 1 | Swap the palettes: [theme.light] needs a light background, [theme.dark] a dark one (sdk/docs/guide/manifest.md#theme). |
| `manifest.theme-unused` | A `[theme]` that cannot be used; the pages show tessoa's colours. | 1 | Write the colours under [theme.light] and [theme.dark] as sdk/docs/guide/manifest.md#theme shows. |
| `manifest.theme-key-ignored` | A key in `[theme]` that is ignored. | 1 | Remove the key or move it under [theme.light] or [theme.dark] (sdk/docs/guide/manifest.md#theme). |
| `manifest.page-icon-unknown` | A page icon tessoa does not know. | 1 | Use an icon name from sdk/docs/guide/manifest.md#pages, or the path of an image file in the project. |
| `manifest.translation-ignored` | A text in another language that is ignored. | 1 | Use a language tessoa knows, with a non-empty text (sdk/docs/guide/manifest.md#texts-in-other-languages). |
| `manifest.setting-ignored` | A setting, or a key of one, that is ignored. | 1 | Fix the setting as the message says; sdk/docs/guide/manifest.md#settings lists the keys of each kind. |
| `manifest.ignored` | Anything else in the manifest that is ignored. | 1 | Fix or remove what the message names; sdk/docs/guide/manifest.md describes every key. |
| `audit.undeclared-permission` | The component calls a function whose permission the manifest does not declare. | 1 | Declare the permission under [permissions] in plugin.toml (sdk/docs/guide/permissions.md#names), or stop calling the function. |
| `audit.unknown-function` | The component calls a function this interface version does not define. | 1 | Call only functions listed in sdk/docs/api/; run tessoa-plugin upgrade if the kit is older than plugin.toml's api. |
| `audit.unsupported-import` | The component imports an interface the runner does not provide, such as the file system or sockets. | 1 | Remove the code that uses files, sockets or other system interfaces directly; ask tessoa through the plugin interface instead (sdk/docs/guide/permissions.md#what-the-plugin-never-gets). |
| `audit.pages-not-exported` | The manifest declares pages and the component does not export them. | 1 | Build for the world tessoa-plugin-pages: export the pages (sdk/docs/guide/pages.md), or remove [[pages]] from plugin.toml. |
| `audit.no-websites` | The component uses the network and the manifest lists no website. | 1 | List the websites under [permissions] net in plugin.toml (sdk/docs/guide/network.md#declare-the-websites). |
| `load.refused` | The runner does not load the plugin. | 1 | The runner would not load the plugin; fix what the message names, usually the manifest, the audit or the build. |
| `build.not-built` | The entry file is missing, with `--no-build` or for `check`. | 1 | Run tessoa-plugin build, or leave out --no-build. |
| `build.no-cargo-toml` | `entry.kind` is `wasm` and there is no `Cargo.toml`. | 1 | A Rust plugin needs a Cargo.toml beside plugin.toml; with TypeScript, set entry.kind = "js". |
| `build.no-package-json` | `entry.kind` is `js` and there is no `package.json`. | 1 | A TypeScript plugin needs a package.json beside plugin.toml; with Rust, set entry.kind = "wasm". |
| `build.cargo-failed` | cargo fails to build the component. | 1 | Fix the compiler errors in the message, then build again. |
| `build.npm-install-failed` | `npm install` fails. | 1 | Read npm's output above; check the network and package.json, then build again. |
| `build.npm-failed` | `npm run build` fails. | 1 | Fix the errors npm run build printed, then build again. |
| `build.no-output` | The build ends without writing the entry file. | 1 | Make the build script write the file entry.file in plugin.toml names. |
| `test.no-scenarios` | No scenario in the directory, or none matches the names given. | 2 | Add scenarios to tests/scenarios (sdk/docs/guide/testing.md#a-scenario), or check the names given to filter them. |
| `test.scenario-invalid` | A scenario file cannot be read. | 1 | Fix the scenario file as the message says; sdk/docs/guide/testing.md lists its keys. |
| `test.scenario-failed` | A scenario's expectations are not met. | 1 | Compare what the scenario expects with what the plugin did; the failure names both (sdk/docs/guide/testing.md#expect). |
| `test.step-failed` | A step of a page scenario fails. | 1 | Read the pages printed with the failure; fix the plugin or the step (sdk/docs/guide/testing.md#page-scenarios). |
| `test.row-off-screen` | A step uses a row of a list that is not on its screen. | 1 | Scroll first: add a step scroll = { on = "<list>", row = <number> } before the one that uses the row (sdk/docs/guide/testing.md#actions). |
| `render.failed` | `render` cannot show the page. | 1 | Fix what the message names; render needs a built plugin and a page plugin.toml declares. |
| `sandbox.trap` | The plugin crashed. | 1 | The plugin crashed; the detail shows where. Fix the code, then run it again. |
| `sandbox.limit` | The runner stopped the plugin: too long without returning, too much CPU time or memory. | 1 | The plugin ran too long or asked for too much memory; do less per call and move long work into a task (sdk/docs/guide/tasks.md). |
| `run.plugin-error` | The command returned an error. | 1 | The plugin returned an error; its message says why. |
| `install.refused` | The running tessoa did not take the folder. | 1 | tessoa is running but did not take the folder; try again, or restart tessoa. |
| `install.not-project` | tessoa says the folder is not a plugin project. | 2 | tessoa found no plugin.toml in the folder; pass the project directory with --dir. |
| `kit.signature` | The kit manifest's signature does not verify. | 1 | The kit manifest is not signed by the tessoa key; nothing was changed. Try again later, and do not override the key. |
| `kit.hash` | A download does not match its size or sha256. | 1 | A download does not match the size or sha256 the manifest names; nothing was changed. Try again. |
| `kit.schema` | The kit manifest is not one this tool reads. | 1 | The kit manifest is newer than this tool understands; update tessoa-plugin with the template's setup script. |
| `kit.archive` | The kit archive cannot be unpacked or says it is another version. | 1 | The kit archive is not usable; nothing was changed. Try again, or pick another version with --version. |
| `kit.version` | `upgrade --version` names a kit the manifest does not list. | 1 | Pick a version the manifest lists, or leave out --version for the newest. |
| `kit.needs-newer-tool` | The kit needs a newer tool than the one running. | 1 | Update tessoa-plugin first: run the template's setup script again, then tessoa-plugin upgrade. |
| `kit.tool-update` | The tool could not update itself; the kit may still be updated. | 1 | The tool could not replace itself; close other copies of tessoa-plugin and run tessoa-plugin upgrade again. |
| `kit.swap` | `sdk/` cannot be replaced. | 1 | sdk/ could not be replaced; close programs that use files in it and run tessoa-plugin upgrade again. |
| `net.fetch` | A download fails. | 1 | Check the network connection and any proxy, then try again. |
| `internal` | Anything the tool did not expect. | 1 | Unexpected; run the command again, and report the message if it persists. |

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
- `new` writes the skeleton it carries, and takes the kit from the same signed
  manifest under `download.tessoa.com` that `upgrade` reads: the kit of the
  tool's own version, or the newest older one when there is none. Without a
  network connection, pass `--kit` with a kit zip as `upgrade` downloads it, or
  with the `sdk/` folder of another project.
