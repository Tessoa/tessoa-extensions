<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# task

What a worker uses while it runs a task. Outside a worker the functions do
nothing, and `cancelled` is false.
Available without declaring a permission.

## Functions

### progress

```
wit  progress: func(done: f64, total: f64, text: string)
rust tessoa_plugin_sdk::task::progress(done: f64, total: f64, text: &str)
ts   tessoa.task.progress(done: number, total: number, text?: string): void
```

Report progress: `done` of `total`, and a line of text for tessoa's
transfer panel; `total` 0 means the total is not known. Report after
every item: a worker that does not call the host for 60 seconds is
stopped. tessoa shows at most 10 reports a second.

Permission: none.

### cancelled

```
wit  cancelled: func() -> bool
rust tessoa_plugin_sdk::task::cancelled() -> bool
ts   tessoa.task.cancelled(): boolean
```

Whether the task was cancelled. A worker checks between items, cleans
up and returns.

Permission: none.
