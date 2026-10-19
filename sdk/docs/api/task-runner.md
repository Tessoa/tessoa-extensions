<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# task-runner

What a worker runs. Every plugin exports it; a plugin without tasks
answers that it has none.

Every plugin exports this interface. The runner calls it in a worker: a second instance of the plugin, started for each task. In Rust, implement `tessoa_plugin_sdk::Task` for each kind of task and pair the kinds with the types in `tessoa_plugin_sdk::export!` or `export_pages!`, as `tasks: [("hash", Hash)]`. In TypeScript, call `tessoa.tasks.define(kind, run)` for each kind when the script loads. A plugin without tasks answers that it has none.

## Functions

### run

```
wit  run: func(kind: string, input: list<u8>) -> result<list<u8>, string>
rust fn run(&mut self, input: Vec<u8>, task: &TaskCtx) -> Result<Vec<u8>, String>   (in impl Task, paired with its kind in export!)
ts   tessoa.tasks.define(kind, async (input, task) => result)   (input and result are JSON)
```

Run a task of `kind` with the `input` given to `tasks.start`. The
result (at most 1 MiB) or the error reaches the plugin's own instance
as `task-done`. A worker may wait: `pick-files`, `create-file`,
`net.fetch` and `ui.confirm` wait for their answer here. It cannot
touch pages.
