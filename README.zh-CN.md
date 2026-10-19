# tessoa 官方扩展

[English](README.md)

文件管理器 tessoa 的官方扩展，也就是 tessoa 扩展市场里上架的插件。每个插件是 `plugins/` 下一个
完整的插件项目，依赖仓根 `sdk/` 里的 SDK 套件。

| 目录 | id | 语言 |
|---|---|---|
| `plugins/calculator` | `com.tessoa.calculator` | TypeScript |
| `plugins/rename-by-date` | `com.tessoa.rename-by-date` | TypeScript |
| `plugins/csv-table` | `com.tessoa.csv-table` | TypeScript |
| `plugins/json-tree` | `com.tessoa.json-tree` | TypeScript |
| `plugins/kanban` | `com.tessoa.kanban` | TypeScript |
| `plugins/hash-files` | `com.tessoa.hash-files` | Rust |
| `plugins/image-size-column` | `com.tessoa.image-size-column` | Rust |

每个 `plugin.toml` 旁边有一份 `catalog.toml`，是该插件在扩展清单里的文案：中英文名称、一句话说明，
以及要求的最低 tessoa 版本。

## 装环境

两种语言都用到，装环境脚本各跑一次：

```
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang ts     # Windows
powershell -ExecutionPolicy Bypass -File scripts/setup.ps1 -Lang rust
bash scripts/setup.sh --lang ts                                      # macOS、Linux
bash scripts/setup.sh --lang rust
```

脚本只装缺的：Node.js、带 `wasm32-wasip2` 目标的 Rust，以及命令行工具 `tessoa-plugin`。
各工具的版本和每个下载文件的 sha256 记在 `scripts/cli.lock`。

## 构建

```
scripts/build-all.sh               # 全部插件
scripts/build-all.sh kanban        # 只构建指定的
```

对每个插件先跑 `tessoa-plugin test`，再跑 `tessoa-plugin package`，遇到第一个失败即停止。
Windows 上在 Git Bash 里运行。`tessoa-plugin` 不在 PATH 上时，用环境变量 `TESSOA_PLUGIN` 指定。

单个插件也可以单独处理，在它的目录里或加 `--dir plugins/<名>`：`tessoa-plugin test`、
`tessoa-plugin package`，以及用 `tessoa-plugin install` 装进本机正在运行的 tessoa 试用。

## 发布包

`scripts/build-all.sh` 把发布包写到仓根的 `dist/<id>-<version>.zip`，最后列出每个包的 sha256
和大小，供扩展清单填写。版本号取自 `plugin.toml`，每次发布前在那里调高。`dist/` 是构建产物，不提交。

## SDK 套件

`sdk/` 是插件模板附带的套件，不要改动；要换新版，在仓根运行 `tessoa-plugin upgrade`，然后重新构建。
说明书是 `sdk/AGENTS.md`，清单字段见 `sdk/docs/guide/manifest.md`。
