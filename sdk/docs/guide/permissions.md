# Permissions and the sandbox

A plugin runs inside a WebAssembly sandbox. Out of the box it can compute,
keep memory, read the clock and draw random numbers. Everything else goes
through the host interface, and the parts of the interface that touch the
user's data or leave the computer are gated by permissions declared in the
manifest.

## Levels

| Level | How it is granted | What is in it |
|---|---|---|
| Automatic | Nothing to declare. | Logging, notifications, dialogs, the interface language, the metadata of the selected files (name, size, time, folder or not), the plugin's own storage, pages, describing and releasing kept files, the file and folder pickers, settings, images, tasks, offering a web address (tessoa asks the user every time). |
| Declared | Listed in `[permissions]`; shown to the user when the plugin is installed. | Reading the files the user hands to the plugin or that the file list shows; renaming the selected files; keeping files, copying, moving, opening and deleting kept files; creating files in a folder the user picks; writing to the clipboard; connecting to the websites the manifest lists. |
| Consented | Listed in `[permissions]` and confirmed by the user inside tessoa, at installation and again when an update asks for more. | Reserved for later versions: reading a whole folder, changing the content of existing files. |
| Official | Not available to third-party plugins. | Native plugins shipped by tessoa itself. |

Consent is given by clicking in tessoa. No command-line tool in this
repository can grant a permission, and the test host grants only what a test
author writes down for that test.

## Names

| Permission | Gates | What the plugin can then do |
|---|---|---|
| `files.read-selected` | `files.read`, `refs.read` | Read the content of the files the user hands to it: the ones selected for a command, dropped on a page, or picked in tessoa's file picker, and the files it kept or created. |
| `files.read-listed` | `files.read` in a column | Read the content of the files tessoa's file list shows, to compute its columns, only while the user shows such a column. See [columns.md](columns.md). |
| `files.rename-selected` | `files.rename` | Ask tessoa to rename the selected files. tessoa applies its conflict handling, the rename is undoable and appears in the activity log. |
| `files.keep` | `files.keep` | Keep a selected file as a reference that stays valid after the command and after tessoa restarts. tessoa keeps the path; the plugin sees a number. Files dropped on a page and files picked become references without this permission: the user handed them over. |
| `files.transfer` | `refs.copy-to-picked`, `refs.move-to-picked` | Copy or move kept files to a folder the user picks, with tessoa's progress, conflict handling and undo. |
| `files.open` | `refs.open`, `refs.reveal` | Open a kept file as a double click would, or show it in a new tab. |
| `files.delete` | `refs.trash` | Move kept files to the recycle bin. tessoa asks the user every time, in its own dialog that names the plugin. |
| `files.create` | `files.create-file`, `files.create-file-start` | Create new files in a folder the user picked, through tessoa, with undo; never replacing a file. See [files.md](files.md). |
| `clipboard.write` | `clipboard.write-text` | Put text on the clipboard. |

In the manifest a permission is written as category and item:

```toml
[permissions]
files = ["read-selected", "rename-selected", "keep"]
clipboard = ["write"]
```

## Websites

`net` lists the websites the plugin connects to, rather than a permission
name; every function of `net` needs at least one. The installation shows the
user each website. See [network.md](network.md).

```toml
[permissions]
net = ["https://api.example.com"]
```

## What the user is told

The installation lists every permission and website in plain words. Two
lines more appear when they apply:

- A plugin with `[[columns]]` sees the names of the files in the folders the
  user opens, for its columns.
- A plugin that can read files (`files.read-selected` or
  `files.read-listed`) and connect to websites can send what it reads to
  those websites. Ask for both only when the plugin needs both.

## What the plugin never gets

- Paths. The plugin receives handles that are valid for one command or one
  request of a column, and references to the files the user handed to it, as
  numbers.
- Files it was not given. A handle from an earlier command, or a guessed one,
  is refused with `not-granted`, and so is a reference or a picked folder
  that was not issued to the plugin.
- The file system or the network through WASI. The runner does not link those
  interfaces; a component that imports them is refused at installation and
  cannot be instantiated. Connections go through `net`, to the listed
  websites only.
- Environment variables, command-line arguments, a current directory.

## Checks, in order

1. **Installation.** The installer reads the component's import table. A call
   to a gated function without the matching declaration, or an import of an
   interface the runner does not provide, refuses the plugin before it runs.
2. **Call time.** The runner checks the handle and the permission on every
   call, the website of every request and every hop of its redirects, and
   validates every new file name. This is the only check that applies to
   script plugins, whose engine component necessarily imports the whole
   interface.
3. **Limits.** Memory, CPU time per burst and per call, read size, page size,
   storage, images, requests and tasks are capped; see
   [the runner design](../design/runner.md) and the guides of each.

Ask for the fewest permissions that work. The examples declare exactly what
they use, and the tests check it: a Rust example against what its calls
need, a TypeScript one by taking each permission away and seeing a scenario
fail.
