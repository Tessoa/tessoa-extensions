<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# plugin

What every plugin exports.

A plugin exports this interface. In Rust, implement `tessoa_plugin_sdk::Plugin` and pass the type to `tessoa_plugin_sdk::export!`. In TypeScript, export `run` from the module bundled with global name `plugin`, or assign `globalThis.plugin = { run }`.

## Functions

### run-command

```
wit  run-command: func(command-id: string) -> result<_, string>
rust fn run_command(command_id: &str) -> Result<(), String>   (in impl Plugin)
ts   plugin.runCommand(commandId: string): void | Promise<void>
```

Run one of the commands declared in the manifest. The error text goes
to the plugin log and is shown to the user.

### on-event

```
wit  on-event: func(ev: plugin-event)
rust fn on_event(event: PluginEvent)   (in impl Plugin, optional; every open page hears the same as an Event, and the pages are rendered again afterwards)
ts   nothing to write: a task's handle follows its progress and outcome, and every page renders again
```

Something happened; see `plugin-event`. Called in the plugin's own
instance, never in a worker.

## Types

### task-progress

Defined in [tasks](tasks.md#task-progress).

### task-outcome

Defined in [tasks](tasks.md#task-outcome).

### plugin-event

Variant; Rust `PluginEvent`, TypeScript `PluginEvent`.

Something that concerns the plugin as a whole rather than one page.

| Case | Payload | Meaning |
|---|---|---|
| `task-progress` | `task-progress` | A task reported progress; at most 5 times a second per task. |
| `task-done` | `tuple<u64, task-outcome>` | A task ended. Carries the task id and the outcome. |
| `settings-changed` | `list<string>` | The user changed settings. Carries their ids. |
