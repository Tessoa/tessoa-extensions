# Tasks

Work that takes longer than a second runs as a task: hashing a hundred
files, converting images, building a report, fetching many pages. A task
runs in a worker, a second instance of the plugin, while the plugin's pages
and commands go on working. tessoa shows every task in its transfer panel,
with the plugin's name, the task's title, its progress and a button that
cancels it.

A page that computes for long in a handler holds itself up and is stopped
after a while; a command that loops is stopped too. Rule of thumb: when the
work goes item by item over something the user chose, or waits for a
website, make it a task.

`examples/hash-files-rs` is a complete one: a command starts a task that
hashes the selected files a piece at a time, reports its progress, stops
when the user cancels it, and then asks for a folder to save the report in.

## A task in TypeScript

Define each kind of task when the script loads, and start it from a page or
a command:

```ts
tessoa.tasks.define("hash", async (input: { files: number[] }, task) => {
  const sums: string[] = [];
  for (const [i, file] of input.files.entries()) {
    if (task.cancelled) return null;                  // a property, not a call
    sums.push(hash(tessoa.refs.readAll(file)));
    task.progress(i + 1, input.files.length, `${i + 1} of ${input.files.length}`);
  }
  return { sums };
});
```

```tsx
// In a page: start it, keep the handle, show how far it is.
<button key="hash" onClick={() => { s.task = tessoa.tasks.start("hash", "Hash files", { files: s.files }); }}>
  Hash
</button>
<progress key="how-far" value={s.task?.progress} />
<text key="state">{s.task ? `${s.task.state}: ${s.task.report.text}` : ""}</text>
```

- `tessoa.tasks.start(kind, title, input)` starts a task of a kind `define`
  named and returns its handle at once. The input goes to the worker as JSON,
  at most 1 MiB; the title is shown in tessoa's transfer panel after the
  plugin's name. It throws an `Error` with the reason when the task cannot
  start.
- The handle follows the task: `state` (`"running"`, then `"done"`,
  `"failed"` or `"cancelled"`), `progress` (0 to 1, `undefined` while the
  total is not known, which `<progress>` shows as a moving bar, and 1 once
  done), `report` (the last `{ done, total, text }` that reached the page),
  `result`, `error`, the promise `done` and `cancel()`. Every page renders
  again whenever a task moves on, so a page reads the handle in `render` and
  needs nothing else.
- The worker's function receives the input, parsed, and the task: report with
  `task.progress(done, total, text)` (`total` 0 when it is not known), look
  at `task.cancelled` between items. What it returns, written as JSON (at
  most 1 MiB), is the result; what it throws is the reason it failed.

## A task in Rust

A task is a type that implements `Task`, listed in the export with the kind
that names it; `tasks::start` starts one.

```rust
struct Hash;

impl Task for Hash {
    fn run(&mut self, input: Vec<u8>, task: &TaskCtx) -> Result<Vec<u8>, String> {
        let files = decode(&input)?;
        let mut sums = Vec::new();
        for (i, file) in files.iter().enumerate() {
            if task.cancelled() {
                return Ok(Vec::new());
            }
            sums.push(hash(&refs::read_all(*file).map_err(|e| format!("{e:?}"))?));
            task.progress((i + 1) as f64, files.len() as f64, &format!("{} of {}", i + 1, files.len()));
        }
        Ok(encode(&sums))
    }
}

tessoa_plugin_sdk::export_pages!(Hasher => "hasher"; tasks: [("hash", Hash)]);
```

- `tasks::start(kind, title, &input)` returns the task's id at once. The
  expression the export pairs with the kind makes a new value for every run;
  a kind the export does not list fails with the list it has.
- How far the task is and how it ended arrive as `Event::TaskProgress { task,
  done, total, text }` and `Event::TaskDone { task, outcome }` in every open
  page, after `Plugin::on_event`. A page knows its own tasks by the id
  `start` returned; `outcome` is `TaskOutcome::Done(bytes)`, `Failed(why)`
  or `Cancelled`.
- A plugin without pages hears the same in `Plugin::on_event(PluginEvent)`.
- `export!(MyPlugin; tasks: [...])` is the same for a plugin without pages.

## Progress

Report after every item: a worker that does not call tessoa for 60 seconds
is stopped, and `progress` is the cheapest call there is. tessoa's transfer
panel shows at most 10 reports a second; the plugin hears at most 5 a second
of each task, and in between only the newest report waits. A page therefore
shows a task's progress a little behind, and the handle's `report` (Rust: the
last `TaskProgress` the page heard) may stay on the last report that reached
it rather than the very last one the worker made: show the end from the
task's state, not from its last report. In TypeScript `progress` is 1 once
the task is done. A report's text longer than 1 KiB is cut.

## Cancelling

The user cancels with the button in the transfer panel; the plugin with
`task.cancel()` on the handle, `tessoa.tasks.cancel(id)`, or Rust
`tasks::cancel(id)`. The worker's `task.cancelled` (Rust
`task.cancelled()`) turns true: look between items, clean up and return.
A worker that has not returned 2 seconds later is stopped. A task still
waiting for its turn ends at once. A cancelled task ends as cancelled
whatever its function returns.

## A page that waits for a task

A page's handler may `await task.done`:

```tsx
onClick={async () => {
  s.task = tessoa.tasks.start("hash", "Hash files", { files: s.files });
  const { sums } = await s.task.done;
  s.sums = sums;                          // the page shows them in its next render
}}
```

The handler stops at the `await` and goes on when the task ends, which is
not something the user did. From there on it can change the state, start and
cancel tasks, store and log, but it can no longer wait for the user or the
network: `tessoa.ui.confirm`, the file pickers, `createFile` and
`tessoa.net.fetch` throw "cannot wait here: this runs because a task moved on
or ended". Two ways that work:

- Ask before the task starts, or let the user ask afterwards: the page shows
  the result with an "Export…" button, and that button's handler picks a
  folder.
- Do the waiting in the task: a worker may wait for the user and the network
  (see below), so a task can pick the folder and write the report itself.

`done` rejects with a `TaskError` whose `kind` is `"failed"` or
`"cancelled"`. A handler that awaits a task the user cancels ends there,
quietly, without a line in the log; a failure goes to the log unless the
handler catches it. A command cannot await a task: the task ends after the
command returns, so the command starts it and leaves it.

## What a worker can do

The worker is another instance of the plugin. It shares the plugin's storage,
file references, settings, the images it added and the folders the user
picked, but not its variables: what the task needs from a page or a command
goes into the input.

- It may wait: `await tessoa.ui.confirm(...)`, the file pickers,
  `createFile` and `tessoa.net.fetch` wait for their answer here (Rust:
  `ui::confirm`, `files::pick_files`, `files::pick_folder`,
  `files::create_file`, `net::fetch`). A question shows over tessoa with the
  plugin's name; the task waits until the user answers.
- A task a command starts reads the command's files with `files.read` until
  it ends; a task a page starts gets file references in its input and reads
  them with `refs.read`.
- It cannot touch pages (no `pagesHost`, no page events), start other
  tasks, rename files or offer web addresses; those calls fail and say so.

## Limits and lifetime

| | |
|---|---|
| Tasks of a plugin | 4 run at once, 64 more wait their turn; beyond that `start` fails: let one task work through many items |
| Input, result | at most 1 MiB each; a larger result fails the task |
| Kind, title | at most 1 KiB each |
| A worker | 256 MiB of memory; no limit on its time, but it is stopped after 60 seconds without calling tessoa |

A task belongs to the plugin, not to the page that started it: closing the
page does not stop it. It stops when the plugin stops, is disabled, is
updated or is reloaded during development, and when tessoa quits; tessoa does
not start it again later. A task that should survive keeps its progress in
the storage and the page offers to go on.

## Testing

In a scenario, tasks start when the plugin starts them and run only while a
step waits for them, so every step sees what its own action did:
`wait-tasks = true` runs them to their end, `cancel-task = 1` cancels the
first one the plugin started and waits until it has stopped, `expect-task`
checks a task's state, and after a command the tasks it started run to their
end before `[expect]` is checked, where each must end done unless
`[expect] tasks` says otherwise. See [testing.md](testing.md).

```toml
[[steps]]
click = "hash"

[[steps]]
wait-tasks = true
expect-task = { task = 1, state = "done" }
expect-text = { key = "state", contains = "done" }
```
