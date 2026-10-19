<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# tasks

Long tasks. A task runs in a second instance of the plugin, a worker,
while the plugin's pages keep working; the worker's `task-runner.run`
receives the input and returns the result. The worker shares the plugin's
storage, file references, settings, images and picked folders, but not its
variables: whatever it needs goes into the input. tessoa shows every task
in its transfer panel, where the user can cancel it.
Available without declaring a permission.

## Functions

### start

```
wit  start: func(kind: string, title: string, input: list<u8>) -> result<u64, string>
rust tessoa_plugin_sdk::tasks::start(kind: &str, title: &str, input: &[u8]) -> Result<u64, String>
ts   tessoa.tasks.start(kind: string, title: string, input?: unknown): Task
```

Start a task of the given kind: a worker's `run` receives `kind` and
`input` (at most 1 MiB). `title` is shown in tessoa's transfer panel
after the plugin's name. Returns the task id; progress and the outcome
arrive through `plugin.on-event`. At most 4 tasks of a plugin run at
once, and the others wait their turn. A task started by a command can
use the files the command was given until it ends. Not available in a
worker.

Permission: none.

### cancel

```
wit  cancel: func(task: u64)
rust tessoa_plugin_sdk::tasks::cancel(task: u64)
ts   tessoa.tasks.cancel(task: Task | number): void
```

Cancel a task: `task.cancelled` turns true in its worker, and a worker
that has not returned 2 seconds later is stopped. A task that has
ended, or that the plugin did not start, is left alone.

Permission: none.

TypeScript also has:

```
tessoa.tasks.define(kind: string, run: (input: any, task: TaskContext) => unknown): void
```

## Types

### task-progress

Record; Rust `TaskProgress`, TypeScript `TaskProgress`.

How far a task is, as the worker reported it with `task.progress`.

| Field | Type | Meaning |
|---|---|---|
| `task` | `u64` |  |
| `done` | `f64` |  |
| `total` | `f64` | 0 when the total is not known. |
| `text` | `string` |  |

### task-outcome

Variant; Rust `TaskOutcome`, TypeScript `TaskOutcome`.

How a task ended.

| Case | Payload | Meaning |
|---|---|---|
| `done` | `list<u8>` | `run` returned these bytes. |
| `failed` | `string` | `run` returned an error, or the worker stopped. Carries the reason. |
| `cancelled` | none | The user or the plugin cancelled the task. |
