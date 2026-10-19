# Plugins in TypeScript or JavaScript

A script plugin is a single JavaScript file that the JavaScript engine
component runs inside the sandbox. The engine is part of the runner; the
plugin ships only its script. Same sandbox, same rules, same interface as a
Rust plugin.

## Shape

```
my-plugin/
  plugin.toml          entry.kind = "js", entry.file = "dist/plugin.js"
  package.json         build script, dev dependencies
  tsconfig.json        "types": ["@tessoa/plugin-sdk"], the JSX settings
  src/plugin.ts        export async function run(commandId: string)
                       (src/plugin.tsx for a plugin with pages)
  tests/scenarios/     see testing.md
  tests/fixtures/
```

The build checks the types, then bundles `src/plugin.ts` or
`src/plugin.tsx` into one file whose exports land on a global named `plugin`:

```
tsc --noEmit && esbuild ./src/plugin --bundle --format=iife --global-name=plugin --target=es2020 --jsx=automatic --jsx-import-source=@tessoa/plugin-sdk --outfile=dist/plugin.js
```

The bundler alone does not check types, so without the first half a call to a
function that does not exist would only fail when the plugin runs. Written as
`./src/plugin`, without an extension, the entry is whichever of the two files
exists; keep only one of them.

## JSX

Pages are written in JSX ([pages.md](pages.md)). Two settings make it work,
and both are already in a project made from the template:

- `tsconfig.json`: `"jsx": "react-jsx"` and `"jsxImportSource":
  "@tessoa/plugin-sdk"` in `compilerOptions`, and `"src/**/*.tsx"` in
  `include`.
- the build: `--jsx=automatic --jsx-import-source=@tessoa/plugin-sdk` for
  esbuild.

They change nothing for a plugin without JSX. The JSX runtime is the SDK's
own, about twenty lines that turn elements into plain objects; it is not
React and needs no DOM. Element names are the node kinds: `<column>`,
`<text>`, `<button>`, `<text-input>`, `<file-row>` and so on, declared in
`types/pages.d.ts` with every attribute.

The engine calls `plugin.run(commandId)` for every command and waits for the
returned promise. A plain JavaScript file can do the same by assigning
`globalThis.plugin = { run(commandId) { ... } }`; it then needs no build at
all.

## The API

The host interface is the global `tessoa`, one namespace per interface of
`wit/tessoa-plugin.wit`. The declarations in `@tessoa/plugin-sdk` describe
it; in short:

| Call | Notes |
|---|---|
| `tessoa.log.info(message)` and `debug`, `warn`, `error`, `log(level, message)` | Also reachable through `console`. |
| `tessoa.ui.notify(title, body)` | Returns at once. |
| `await tessoa.ui.confirm(title, body, okLabel?, cancelLabel?)` | Resolves to `true` for the ok button. The plugin is suspended while the user decides. |
| `await tessoa.ui.confirmList(title, body, items, okLabel?, cancelLabel?)` | One line per item. |
| `tessoa.files.selected()` | `{ handle, name, size, modifiedMs, isDir }` per selected file. |
| `tessoa.files.read(handle, offset?, maxLen?)`, `readAll(handle)` | `Uint8Array`. Needs `files.read-selected`. |
| `tessoa.files.rename([{ handle, newName }])` | One outcome per request: `{ renamed }`, `{ skipped }` or `{ failed }`. Needs `files.rename-selected`. |
| `await tessoa.files.pickFiles({ title, extensions, multiple })`, `pickFolder({ title })`, `createFile(folder, path, data)` | tessoa's pickers, and a new file in a picked folder. See [files.md](files.md). |
| `await tessoa.net.fetch(url, { method, headers, body })` | A request to a website `plugin.toml` lists; the answer has `ok`, `status`, `text()`, `json()`. See [network.md](network.md). |
| `tessoa.tasks.define(kind, run)`, `tessoa.tasks.start(kind, title, input)` | Long work in a worker, with progress and cancelling. See [tasks.md](tasks.md). |
| `tessoa.settings.get(id)` | A setting `[[settings]]` declares, as its type has it. See [settings.md](settings.md). |
| `tessoa.columns.define(id, (file) => value)` | The values of a column `[[columns]]` declares. See [columns.md](columns.md). |
| `tessoa.images.add(data)` | An image for `<image src={id}>`: the bytes of a picture, or the text of an SVG. |

`read` and `rename` throw a `FilesError` with `kind` set to `not-granted`,
`permission-denied` or `io` when the host refuses the call. An error thrown
out of `run`, or a rejected promise, ends the command with its message in the
log.

For pages:

| Call | Notes |
|---|---|
| `tessoa.pages.define(kind, { open, render, save, title, keys, onKey, onTransferDone, onActivated, close })` | Defines a kind of page declared in `plugin.toml`; call it when the script loads. Handlers change the state in place; what they return is ignored. See [pages.md](pages.md). |
| `tessoa.pages.h(type, props, ...children)` | An element without JSX. |
| `tessoa.pagesHost.open(kind, args?)` | Opens a page of the plugin, or brings a `single` one to the front; returns its id. `args` is any JSON value. |
| `tessoa.pagesHost.setTitle(page, title)` | Changes the title of the page's tab. |
| `await tessoa.ui.confirm(...)` in a page's handler | Asks without holding the page up; the handler goes on with the answer. |
| `tessoa.files.keep(handle)` | A reference to a selected file that outlives the command. Needs `files.keep`. |
| `tessoa.refs.info`, `release`, `copyToPicked`, `moveToPicked`, `trash`, `open`, `reveal` | Work on file references; see the table in [pages.md](pages.md#files). |
| `tessoa.storage.get(key)`, `set(key, bytes)`, `remove(key)`, `keys()` | The plugin's own storage, shared by its pages and commands and kept across restarts. Values are `Uint8Array`; store JSON with `TextEncoder`. |
| `tessoa.clipboard.writeText(text)` | Needs `clipboard.write`. |
| `tessoa.ui.language()` | `en`, `zh`, `ja` or `de`, for the plugin's own texts. |
| `tessoa.ui.openUrl(url)` | Offers a web address; tessoa asks the user before opening it. Only while a handler handles something the user did. |
| `tessoa.ui.openSettings()` | Opens the plugin's settings in tessoa. |
| `new tessoa.pages.TreeModel()` | The rows of a tree; see [lists.md](lists.md#trees). |

## Text

File contents arrive as bytes. `TextDecoder` and `TextEncoder` are the
standard ones, for UTF-8 only:

```ts
const text = new TextDecoder().decode(tessoa.files.readAll(file.handle));
const bytes = new TextEncoder().encode(text);
```

Invalid bytes become U+FFFD, or throw with `{ fatal: true }`; a leading byte
order mark is dropped. When reading a file in chunks with `read`, decode each
chunk with `{ stream: true }` so that a character split between two chunks
comes out whole. Any encoding other than UTF-8 throws a `RangeError`; decode
UTF-16 or legacy code pages yourself if a plugin needs them. Do not add `DOM`
to `lib` in `tsconfig.json` for these: the SDK declares them, and the DOM
declarations would clash.

## What is different from Node.js

There is no `require`, no `import` of Node modules, no `fs`, no global
`fetch` (`tessoa.net.fetch` reaches the websites the manifest lists), no
timers. Pure JavaScript libraries bundle fine; anything that needs Node's
built-in modules does not. The engine is QuickJS: ample for reading headers
and deciding names, but a byte-by-byte loop over megabytes runs a couple of
hundred times slower than the same loop in Rust. Heavy processing belongs in
a Rust plugin, and in a task in either language, so that the pages keep
working meanwhile.

Standard output is captured into the plugin log, so `print` style debugging
works but `console.log` is the intended way.

## Limits

The same as for every plugin: memory, CPU time per burst and per command, read
size. An endless loop is stopped by the runner and reported; the next command
starts a fresh engine.
