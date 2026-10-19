# Writing a tessoa plugin

This file is the manual for whoever writes the plugin in this project, human or
tool. It ships with the SDK kit in `sdk/` and is replaced by `tessoa-plugin upgrade`,
so do not edit it. Notes about this particular project belong at the end of the
`AGENTS.md` at the project root, which upgrades leave alone.

Kit version: see `sdk/VERSION`. Interface: see `api` in `plugin.toml`.

## What a plugin is

tessoa runs each plugin in a separate runner process, inside a WebAssembly sandbox.
The plugin never sees file paths and has no file system, network or process access of
its own. It receives handles for the files the user selected and asks tessoa to act
on them; it reaches only the websites its manifest lists, through the runner. Anything
beyond the basics must be declared in `plugin.toml` and is shown to the user, who
confirms it in tessoa.

The plugins in `examples/` are references to copy from. None of them is the
user's plugin or is installed anywhere: when the user asks for a plugin that an
example already does, build it in this project all the same, starting from that
example.

## Steps

1. Pick the language. TypeScript is enough for most plugins that rename, filter or
   label files. Pick Rust when the task is heavy computation or needs a crate.
2. Run the setup script for that language. On Windows:
   `powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts` (or
   `-Lang rust`); elsewhere: `bash scripts/setup.sh --lang ts` (or `--lang rust`).
   Run it without asking the user first: it is part of the work, installs for
   the current user only, needs no administrator rights, and prints every
   setting it changes. It installs only what is missing, `tessoa-plugin` included, and prints the
   full path of every tool. A terminal that was open before the setup does not
   find what it just installed: when that is the case, the script ends with a
   line that adds the tools to the terminal's PATH. Run that line first. If
   every command you run starts a new shell, start each command that calls
   `node`, `npm`, `cargo` or `tessoa-plugin` with it. `tessoa-plugin` itself
   finds the Node.js and Rust the script installed, so calling it by its full
   path also works.
3. If `skeleton/` still exists, run
   `tessoa-plugin init --lang ts|rust --id <reverse.domain.id> --name "<Name>"`.
   It keeps the skeleton you chose and deletes the other one. Do not set the
   project up by hand instead, by writing `plugin.toml` or copying a skeleton
   or an example to the root.
4. Run `tessoa-plugin doctor`. Fix whatever it marks FAIL before going on; it
   fails while `skeleton/` is left.
5. Declare the commands, pages, columns or settings and their permissions in
   `plugin.toml` (see `sdk/docs/guide/manifest.md`). Ask for the fewest permissions
   that work. The skeleton's `count-selected` command, its code and its scenario are
   placeholders; replace them. For a page, read the Pages section below first; for a
   list, a long piece of work or a website, the section after it.
6. Write the plugin in `src/`.
7. Write scenarios in `tests/scenarios/*.toml` with fixtures in `tests/fixtures`
   (see `sdk/docs/guide/testing.md`): for a command, which files and answers go in
   and what must be true afterwards; for a page, steps that play the user. Logic
   that does not touch the interface can also have ordinary unit tests in the
   plugin's own language.
8. Run `tessoa-plugin test` until everything passes. The first build of a
   TypeScript project runs `npm install` by itself.
9. Describe the plugin in `README.md`: each command and page, where it appears,
   what it does and the permissions it asks for. The package includes this file.
10. Run `tessoa-plugin package`. It writes `dist/<id>-<version>.zip` and refuses a
    plugin that uses a function it did not declare.
11. Run `tessoa-plugin install`. It hands the project to the running tessoa, which
    asks the user to confirm the permissions the first time and reloads the plugin
    after every later build.
12. Tell the user where to find the command or page in tessoa and what they should
    see.

The work is done when the tests pass, the package is built, the plugin is installed
and the user has been told where to try it. If `install` reports that tessoa cannot
be found or is too old, say so; do not work around it.

## Commands

| Command | Use |
|---|---|
| `tessoa-plugin doctor` | Check the toolchain and the project. |
| `tessoa-plugin check` | Report what the manifest gets wrong without stopping the plugin: a theme, a page icon. |
| `tessoa-plugin build` | Build and audit. |
| `tessoa-plugin test` | Run the scenarios. |
| `tessoa-plugin render [--page <id>] [--steps <scenario>] [--rows 100..140]` | Print a page as text, freshly opened or after the steps of a scenario; a list's rows on screen, or those rows. |
| `tessoa-plugin run --command <id> --file <path> --answer ok` | Try one command on real files. The files are really renamed; use copies. |
| `tessoa-plugin package` | Write the zip. |
| `tessoa-plugin install` | Hand the project to tessoa. |
| `tessoa-plugin upgrade` | Replace `sdk/` with the newest kit. Run `--check` to only look. |

There is no option that grants a permission, and none may be invented. Consent is
given only by the user, inside tessoa.

## Rules the sandbox enforces

- No paths, no file system, no threads, no processes, and no network except
  `tessoa.net` (Rust `net`) to the websites `plugin.toml` lists. Do not try to reach
  them through the language's standard library; the runner does not link them.
- Only the functions named in `sdk/docs/api/` exist. Read the page for an interface
  before using it; do not guess names.
- A function used without the matching permission in `plugin.toml` fails the audit
  at build time and the call fails at run time.
- A command or a page's handler that runs long or loops is stopped by the runner.
  Keep it small and do not poll; longer work runs as a task (below).
- In a command, dialogs block until the user answers. Ask once for a batch, not
  once per file. A page never waits for a dialog.
- File changes are requests: tessoa applies its own conflict handling, recycle bin,
  undo and activity log. Do not try to emulate them.

## Pages

A page is a whole tab, or a section of the sidebar, that the plugin fills; tessoa
draws it with its own widgets, in the user's theme and language. Write a page when
the user keeps something open: a tool, a place to collect things, a board. Write a
command for an action on the selected files. A plugin can have both.

- Declare each page under `[[pages]]` in `plugin.toml`. A plugin with pages only
  has no `[[commands]]`: remove the skeleton's command, its code and its scenario.
- TypeScript: rename `src/plugin.ts` to `src/plugin.tsx` and define the page with
  `tessoa.pages.define`; the build and `tsconfig.json` already handle JSX. Rust: add
  `features = ["pages"]` to the `tessoa-plugin-sdk` dependency in `Cargo.toml`,
  implement `Page`, and use `export_pages!` instead of `export!`.
- Read `sdk/docs/guide/pages.md` before writing the page, and start from the
  closest example: `examples/calculator-ts` and `calculator-rs` (buttons, keys, the
  clipboard; the TypeScript one has a theme), `examples/controls-ts` and
  `controls-rs` (every control: choices, values, colour, date, image, rich text,
  grid, fold, popup, split, list, table, tree), `examples/shelf-ts` (files),
  `examples/kanban-ts` (cards to drag, a menu, a question before deleting),
  `examples/csv-table-ts` (a picked file in a table that sorts, rows copied),
  `examples/json-tree-ts` (a picked file as a tree with `TreeModel`),
  `examples/github-info-ts` (a website asked from a page, and its errors).
- `render` describes the whole page every time and changes nothing; handlers
  only change the state. After every event and every command the SDK renders
  every open page again. Do not update another page by hand.
- In TypeScript, change the state object in place: what a handler returns is
  ignored, and there are no hooks such as `useState`. The handlers on elements
  receive no state; they use the one `render` received.
- Give every node the user acts on a key that is unique in the whole page. In a
  list build it from the item's id (`card-12`), never from its position. In Rust
  a node without a key sends no events at all.
- Declare the keys the page handles, written the way they type: `8`, `+`,
  `Enter`, `Ctrl+c`. Write `Ctrl+c` and `Meta+c` for a shortcut.
- A control shows the value the page gives it: a checkbox, a switch, a choice, a
  number, a slider, a colour, a date, a fold, a popup. Keep the user's change in the
  state in `onChange` or `onToggle` (Rust: `Event::Choose`, `Value`, `Toggle` and so
  on), or the control goes back to the old value.
- A text field keeps what the user typed until the page gives it a different
  value. To empty a field after a submit, keep its text in the state from
  `onInput` (Rust: `Event::Input`) and set it to `""`.
- Open a popup from a button the user clicks, and close it in `onDismiss` by
  setting `open` to false. A popup that is open when the page opens covers the page.
- Pictures for `<image src="images/...">` go in the project's `images` folder:
  `tessoa-plugin package` includes that folder, and nothing else of the kind.
- A button or icon button with a `menu` opens the menu on a click and sends no
  click: give a "more" button the menu and no `onClick`.
- Ask the user as little as possible. In a page, `await tessoa.ui.confirm(...)`
  (TypeScript) or `pages::confirm` (Rust) does not hold the page up; the answer
  arrives later, and the data may have changed by then.
- File references come only from the user: files dropped on a drop zone, files
  a command kept with `files.keep`, files picked with `pickFiles`, files created
  with `createFile` in a folder picked with `pickFolder`
  (`sdk/docs/guide/files.md`). Copy, move and trash are requests tessoa carries
  out; the outcome arrives in `onTransferDone` (Rust: `Event::TransferDone`).
- All pages of the plugin share one instance: a variable outside the page state
  is shared by every page and lost when the plugin restarts. Data every page
  shows goes in the storage (`tessoa.storage`, Rust `storage`); what one tab needs
  after a restart is what `save` returns. In TypeScript the saved state is JSON:
  save a `Set` or a `Map` as an array.
- Icons are names from tessoa's set, listed in `sdk/docs/guide/pages.md`; copy one
  from there. TypeScript does not build with a name outside the list; anywhere
  else an unknown name shows a puzzle piece.
- The children that `grow` share the space left equally, whatever their content,
  but none gets less than its content needs.
- A page is at most 64 levels deep; a deeper tree fails the page.
- Give every `draggable` the item's id as its key and as its payload, as
  `examples/kanban-ts` does: a drop names the draggable it landed in front of by
  its key, and without keys every item lands at the end.
- `where = ["sidebar"]` in `[[pages]]` opens the page in the sidebar instead of
  a tab; the first place listed decides.
- Colours: a page shows tessoa's colours unless `plugin.toml` has a `[theme]`
  (see `sdk/docs/guide/pages.md`, Colours). If you write one, run
  `tessoa-plugin check` and fix every `problem:` line it prints.
- Test the page with page scenarios (`[[steps]]` in `tests/scenarios`; see
  `sdk/docs/guide/testing.md`): open the page, click, type, choose, set values,
  select rows, drop, answer, restart, and check the page after each step. Cover
  every flow the user can take, a restart included. A step refuses what the user
  could not do, such as clicking a row that is not on the screen or a node behind
  an open popup: change the steps, not the page.
- `tessoa-plugin render` prints the page as text, one line per node. Run it, with
  `--steps <scenario>`, whenever a check fails or the page may not look as
  intended.

## Lists, long work and websites

Three rules that decide how the plugin is built. Follow them from the start:
changing course later means rewriting the page.

- **More than two hundred items, or items the user selects: a list, a table or a
  tree.** Not a column of nodes. Give the node a key, `count` and `row={(i) => ...}`
  (Rust: `list(n)` and `Page::rows`); tessoa asks only for the rows on screen,
  holds the selection, and tells the page in `onSelect`. Build each row's id from
  the item's own id. Sorting is the page's: sort the data in `onSort` and render
  `sort`. For a tree use `TreeModel`. Read `sdk/docs/guide/lists.md`.
- **Work that takes longer than about a second: a task.** Hashing files,
  converting, fetching many pages, writing a report. Define it with
  `tessoa.tasks.define` (Rust: a `Task` listed in `export!` or `export_pages!`),
  start it with `tessoa.tasks.start`, and show its handle's `progress` and `state`
  on the page. The task runs in another instance of the plugin: pass what it needs
  in its input. Report progress after every item and check `task.cancelled`
  between items. After `await task.done` a page's handler cannot ask the user or
  go to the network any more: ask before, in a later handler, or in the task. Read
  `sdk/docs/guide/tasks.md`; `examples/hash-files-rs` is a command whose task hashes
  the selected files and saves a report in a folder the user picks.
- **A website: list it first.** Put it under `[permissions] net` in `plugin.toml`
  (`net = ["https://api.example.com"]`) before writing the code that calls it; a
  request to anywhere else fails with `not-allowed`. Use `tessoa.net.fetch` (Rust
  `net::fetch`, or `net::fetch_start` in a page). Scenarios never go to the network:
  answer the requests with `[[net]]` entries. A plugin that reads files and connects
  to websites is shown to the user with a warning; ask for both only when it needs
  both. Read `sdk/docs/guide/network.md`; `examples/github-info-ts` asks GitHub's API
  from a page.

Columns for tessoa's file list are in `sdk/docs/guide/columns.md`
(`examples/image-size-column-rs`), settings the user changes in tessoa in
`sdk/docs/guide/settings.md` (`examples/hash-files-rs` has one).

## Where to look

| Need | File |
|---|---|
| Every function, per interface | `sdk/docs/api/README.md` and the pages it links |
| Pages: the nodes, controls, keys, files, both languages | `sdk/docs/guide/pages.md` |
| Lists, tables, trees, rows, selection, `TreeModel` | `sdk/docs/guide/lists.md` |
| Tasks: long work, progress, cancelling | `sdk/docs/guide/tasks.md` |
| Connecting to websites | `sdk/docs/guide/network.md` |
| Columns in tessoa's file list | `sdk/docs/guide/columns.md` |
| Settings tessoa shows and keeps | `sdk/docs/guide/settings.md` |
| Picking files, creating files in a picked folder | `sdk/docs/guide/files.md` |
| The manifest keys | `sdk/docs/guide/manifest.md` |
| Permissions and what each allows | `sdk/docs/guide/permissions.md` |
| Scenario files | `sdk/docs/guide/testing.md` |
| Commands in detail | `sdk/docs/guide/cli.md` |
| TypeScript specifics | `sdk/docs/guide/typescript.md` |
| The interface definition | `sdk/wit/tessoa-plugin.wit` |
| What may be done with the SDK, the command-line tool and the plugins built with them | `sdk/LICENSE.md` |
| Complete working plugins | `examples/` |

Copy from `examples/` rather than from memory: they build and pass their tests
against the kit in `sdk/`. Each one is a project of its own; run its tests with
`tessoa-plugin test --dir examples/<name>`.

## Layout

```
plugin.toml        manifest
src/               your code
tests/scenarios/   scenario files
tests/fixtures/    files the scenarios copy
sdk/               this kit; replaced on upgrade, never edit
examples/          working plugins to copy from
scripts/           the setup scripts
dist/              build output, not committed
```
