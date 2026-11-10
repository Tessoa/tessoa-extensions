# Writing plugins with a coding tool

The kit is laid out so that a coding tool such as Claude Code, Codex or
Cursor can write a whole plugin from a one-sentence request. What the tool
needs is the manual, `sdk/AGENTS.md`: the steps, the commands, the rules the
sandbox enforces and where to look for everything else. Inside a project the
tool finds it through `AGENTS.md` at the root. This page lists the ways a
tool reaches the manual before there is a project, or in one that did not
start from the template.

## The template

A project made from the plugin template, or with `tessoa-plugin new`,
carries:

- `AGENTS.md`, which sends the tool to `sdk/AGENTS.md`, and one-line pointer
  files under the names some tools look for (`CLAUDE.md`, `GEMINI.md`,
  `.github/copilot-instructions.md`) that send them to `AGENTS.md`.
- `.claude/skills/tessoa-plugin/SKILL.md`, a Claude Code skill. It holds the
  manual of the kit it came with, so Claude Code knows how to work on a
  tessoa plugin as soon as the request mentions one. Nothing else depends on
  the folder; delete it if you do not use Claude Code.

The skill's copy of the manual is not updated by `tessoa-plugin upgrade`.
It says so itself: inside a project, `sdk/AGENTS.md` wins where the two
differ. To bring it up to date, run `tessoa-plugin skill install --project`
after the tool has been upgraded.

## `tessoa-plugin skill install`

For a project that did not start from the template, or to have the skill in
every project:

```
tessoa-plugin skill install             # ~/.claude/skills/tessoa-plugin/SKILL.md
tessoa-plugin skill install --project   # .claude/skills/tessoa-plugin/SKILL.md in this project
tessoa-plugin skill install --check     # only report
```

Without `--project` the skill goes into your own skills folder, which every
project sees: `~/.claude/skills/tessoa-plugin/` on macOS and Linux,
`%USERPROFILE%\.claude\skills\tessoa-plugin\` on Windows. With `--project`
it goes into the project the command finds from the current directory (the
nearest folder with a `plugin.toml` or an `sdk/VERSION`), or the one `--dir`
names. The skill is the one the running `tessoa-plugin` was built with; run
the command again after upgrading the tool. Only `SKILL.md` is written.

The skill is in Claude Code's format only, for now. Other tools read
`AGENTS.md` through the pointer files, or the text below.

## llms.txt

`llms.txt` and `llms-full.txt` will be published at
`https://tessoa.cn/plugins/llms.txt` and `https://tessoa.cn/plugins/llms-full.txt`.
The first is an index of the manual, the guides, the API reference and the
examples, with a sentence on each; the second has the full text of all of
them in one file, for a tool that can read a web page but has no project.
Both are generated from the kit of each SDK release.

## When a tool runs `tessoa-plugin`

A tool that runs the command-line tool should pass `--json`: each command
then prints one JSON document with the result and the problems, each with a
code and a hint on how to fix it. See [cli.md](cli.md#machine-readable-output).
A tool that speaks MCP can use the server below instead.

## MCP server

`tessoa-plugin mcp` offers the commands, the manual, the guides, the API
reference and the examples to a coding tool as an MCP server. The tool
starts it and talks to it on stdin and stdout; nothing else is needed, and it
works before there is a project. Configure it in your editor or agent as a
local (stdio) server with the command `tessoa-plugin` and the one argument
`mcp`. Most tools take a JSON entry of this shape, under a name of your
choice:

```json
{
  "mcpServers": {
    "tessoa-plugin": {
      "command": "tessoa-plugin",
      "args": ["mcp"]
    }
  }
}
```

Where that entry goes depends on the tool; see its documentation on MCP
servers. When `tessoa-plugin` is not on the PATH the tool was started with,
give the full path of the executable as `command` (the setup scripts put it
under `~/.tessoa-plugin/bin`).

The tools:

- `new_project`: create a project, as `tessoa-plugin new`.
- `doctor`: check the toolchain and the project.
- `check`: validate the manifest and audit the built plugin.
- `build`: build, then validate.
- `test`: run the scenarios in `tests/scenarios`.
- `render`: show a page as text, optionally after the steps of a scenario.
- `package`: write the zip for installation.
- `install`: hand the project to the running tessoa, which loads the plugin
  and asks you in its own window to confirm its permissions the first time.
- `api_reference`: a page of the API reference, or one function or type.
- `manifest_reference`: the keys of `plugin.toml`, one section at a time.
- `guide`: the manual (`manual`) or a guide, by name.
- `list_examples`: the example plugins, with what each does and the
  permissions it uses.
- `example_source`: the files of an example, one at a time.

Each tool that runs a command takes `dir`, the absolute path of the project
(for `new_project`, of the folder to create): the server is started in a
folder of the editor's choosing, so it never guesses. Its result is the
document `--json` writes ([cli.md](cli.md#machine-readable-output)), as the
tool's structured result, with a short text beside it; a result whose `ok` is
false is marked as an error, and each problem has its code and hint. The
reference tools answer from the documents built into the `tessoa-plugin`
that runs, so they match its release; a project's `sdk/` may be older or
newer. The same documents are offered as MCP resources, under
`tessoa-plugin://docs/...` and `tessoa-plugin://examples/...`.

What the commands print for a person goes to stderr, which most tools show
as the server's log. A build or a test run takes as long as it does on the
command line, and the server answers one request at a time.
