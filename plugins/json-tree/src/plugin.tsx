// JSON viewer: a JSON file as a tree.
//
// "Open JSON…" lets the user pick a file; the page reads and parses it and
// shows the top-level entries as the rows of a tree. An object or an array
// has an expander, and its entries become rows only when the user opens it:
// a TreeModel keeps the items, knows which are open, and answers tessoa's
// requests for rows by number. Opening an item the first time makes
// `expand` return true, and the page sets its children then, so a file of
// any size opens at once.
//
// Selecting a row shows its path and its value next to the tree; the
// buttons below copy them. The page gives the tree the selection it heard,
// so that opening another file clears it. A double click or Enter on an
// object or an array opens it or closes it, as the expander does.
//
// The page follows the selection (`follow = true` in plugin.toml), the way
// tessoa's preview tabs do: tessoa draws a chip above it that says what it
// follows, and the user changes it there. When a .json file is selected in
// the followed group, the page shows it; anything else leaves the page as
// it is, so that selecting a folder or a picture on the way does not empty
// it.
//
// A file the user hands the page themselves, picked with "Open JSON…",
// dropped on the tree from tessoa or another program, or clicked in the
// history, stays: the page stops taking the files selected in the followed
// group, as a pinned preview tab does, until the user chooses another
// binding on the chip. The chip cannot say so (only tessoa changes it), so
// the status line does. A dropped file opens whatever its name; one that is
// not JSON says so and changes nothing.
//
// The column at the right lists the files the page has shown, newest first
// (history.ts says which ones and for how long); clicking one shows it again.
//
// Each row's id is the entry's path, such as `$.plugins[0].id`, which is
// what a scenario names. A row id is at most 128 bytes, so a path longer
// than that gets a number instead.

import { History } from "./history";

interface Item {
  path: string;
  /** The key in its object, or the index in its array. */
  label: string;
  value: unknown;
}

interface Viewer {
  /** The reference of the file shown, when it came as one; to tell a followed file seen again. */
  file: tessoa.files.FileRef | undefined;
  /** What the page follows, as tessoa last said; undefined until it says. */
  following: tessoa.pagesHost.FollowSource | undefined;
  /**
   * The user chose the file shown on the page: the page takes no followed
   * file until the binding is no longer `heldOn` (the chip changed).
   */
  held: boolean;
  /** The binding when the user chose the file, as JSON. */
  heldOn: string | undefined;
  name: string;
  tree: tessoa.pages.TreeModel<Item>;
  /** The selected row's id. */
  selected: string | undefined;
  history: History;
  /** The history entry of the file shown. */
  current: string | undefined;
  status: string;
}

/** What `save` keeps. Version 1.1 and 1.2 saved `file` alone. */
interface Saved {
  /** The kept references of the history, newest first. */
  history?: number[];
  /** The file shown, when it is one of those. */
  file?: number;
  /** The binding the file was chosen on, as JSON, while the page holds it. */
  heldOn?: string;
}

/** Longest row id, in bytes. */
const MAX_ID = 128;

/** The numbers given to paths that are too long to be ids. */
let numbered = 0;

function isContainer(value: unknown): value is object {
  return typeof value === "object" && value !== null;
}

/** A row id for a path. */
function idOf(path: string): string {
  return new TextEncoder().encode(path).length <= MAX_ID ? path : `#${++numbered}`;
}

/** The path of an entry of an object or an array. */
function childPath(path: string, key: string, inArray: boolean): string {
  if (inArray) return `${path}[${key}]`;
  return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key) ? `${path}.${key}` : `${path}[${JSON.stringify(key)}]`;
}

/** The entries of an object or an array, as items of the tree. */
function entries(path: string, value: object): [string, Item][] {
  const inArray = Array.isArray(value);
  return Object.entries(value).map(([key, v]) => {
    const p = childPath(path, key, inArray);
    return [idOf(p), { path: p, label: key, value: v }];
  });
}

/**
 * Put items in the tree, below `parent` or at the top. An object or an array
 * that is not empty gets an expander; its entries are read when it opens.
 */
function place(tree: tessoa.pages.TreeModel<Item>, parent: string | undefined, items: [string, Item][]): void {
  if (parent === undefined) tree.setRoots(items);
  else tree.setChildren(parent, items);
  for (const [id, item] of items) {
    if (isContainer(item.value) && Object.keys(item.value).length > 0) tree.setLazy(id);
  }
}

/** What a row says about its value. */
function preview(value: unknown): string {
  if (Array.isArray(value)) return `[${value.length} ${value.length === 1 ? "item" : "items"}]`;
  if (isContainer(value)) {
    const n = Object.keys(value).length;
    return `{${n} ${n === 1 ? "key" : "keys"}}`;
  }
  const text = JSON.stringify(value);
  return text.length > 80 ? `${text.slice(0, 79)}…` : text;
}

/** Open or close an item; the first time it opens, its children are read. */
function expand(v: Viewer, id: string, open: boolean): void {
  if (v.tree.expand(id, open)) {
    const item = v.tree.get(id);
    if (item && isContainer(item.value)) place(v.tree, id, entries(item.path, item.value));
  }
}

function zh(): boolean {
  return tessoa.ui.language() === "zh";
}

/** Show a JSON text. Returns false, and changes nothing, when it is not JSON. */
function show(v: Viewer, name: string, text: string): boolean {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch (e) {
    v.status = `Not JSON: ${(e as Error).message}`;
    return false;
  }
  v.name = name;
  v.selected = undefined;
  const roots = isContainer(value) ? entries("$", value) : [[idOf("$"), { path: "$", label: "$", value }] as [string, Item]];
  v.tree.clear();
  place(v.tree, undefined, roots);
  v.status = "";
  return true;
}

/**
 * Read and show a file. `keep`: the user handed it to the page, which keeps
 * its reference; otherwise it was followed, and the history keeps its text.
 * Returns whether it shows.
 */
function load(v: Viewer, file: tessoa.files.FileRef, keep: boolean): boolean {
  const [info] = tessoa.refs.info([file]);
  if (info === undefined) return false;
  let text: string;
  try {
    text = new TextDecoder().decode(tessoa.refs.readAll(file));
  } catch (e) {
    v.status = `Cannot read the file: ${(e as Error).message}`;
    return false;
  }
  if (!show(v, info.name, text)) return false;
  v.file = file;
  v.current = v.history.note(info, keep ? file : undefined, keep ? undefined : text)?.id;
  return true;
}

/** The user chose what the page shows: it holds it until the chip changes. */
function hold(v: Viewer): void {
  v.held = true;
  v.heldOn = JSON.stringify(v.following);
}

/** Show a file the user handed the page, and hold it. */
function choose(v: Viewer, file: tessoa.files.FileRef): void {
  if (load(v, file, true)) hold(v);
  else if (!v.history.entries.some((e) => e.ref === file)) tessoa.refs.release([file]);
}

async function open(v: Viewer): Promise<void> {
  const [file] = await tessoa.files.pickFiles({ title: "Open a JSON file", extensions: ["json"] });
  if (file === undefined) return;
  choose(v, file);
}

/** Open the first file dropped on the page; the others are not kept. */
function drop(v: Viewer, files: tessoa.files.FileRef[]): void {
  const file = tessoa.refs.info(files).find((i) => !i.isDir)?.ref;
  const unused = files.filter((f) => f !== file && !v.history.entries.some((e) => e.ref === f));
  if (unused.length > 0) tessoa.refs.release(unused);
  if (file === undefined) {
    v.status = zh() ? "拖入的不是文件。" : "That is not a file.";
    return;
  }
  choose(v, file);
}

/** Show a file of the history again, and hold it. */
function reopen(v: Viewer, id: string | undefined): void {
  const entry = v.history.get(id);
  if (entry === undefined || entry.id === v.current) return;
  if (entry.ref !== undefined) {
    if (load(v, entry.ref, true)) hold(v);
  } else if (entry.text !== undefined && show(v, entry.name, entry.text)) {
    v.file = undefined;
    v.current = entry.id;
    v.history.entries.splice(v.history.entries.indexOf(entry), 1);
    v.history.entries.unshift(entry);
    hold(v);
  }
}

/**
 * Show the .json file selected in the followed group, unless the page holds
 * a file the user chose and the binding is still the one it was chosen on.
 */
function follow(v: Viewer, f: tessoa.pagesHost.FollowState): void {
  v.following = f.source;
  if (v.held) {
    if (JSON.stringify(f.source) === v.heldOn) return;
    v.held = false;
  }
  if (f.file === undefined || f.file === v.file) return;
  const [info] = tessoa.refs.info([f.file]);
  if (info === undefined || info.isDir || !/\.json$/i.test(info.name)) return;
  load(v, f.file, false);
}

/** What the empty tree says: where to select a file, in tessoa's language. */
function emptyText(v: Viewer): string {
  const zh = tessoa.ui.language() === "zh";
  const s = v.following;
  if (s === undefined) return zh ? "打开或拖入一个 JSON 文件，在这里查看。" : "Open or drop a JSON file to see it here.";
  if ("pinned" in s) {
    return zh ? "已钉住。打开或拖入一个 JSON 文件，在这里查看。" : "Pinned. Open or drop a JSON file to see it here.";
  }
  if ("lost" in s) {
    return zh ? "跟随的组已关闭，请在上方另选一个。" : "The group this page followed has closed; choose another one above.";
  }
  let where: string;
  if ("group" in s) where = zh ? `组 ${s.group}` : `group ${s.group}`;
  else if ("windowGroup" in s) {
    const [w, g] = s.windowGroup;
    where = zh ? `窗 ${w} · 组 ${g}` : `group ${g} of window ${w}`;
  } else if ("tab" in s) where = zh ? "跟随的标签页" : "the tab this page follows";
  else where = zh ? "焦点所在的组" : "the group with the focus";
  return zh ? `在${where}里选一个 .json 文件，或打开、拖入一个。` : `Select a .json file in ${where}, or open or drop one.`;
}

/** The status line: what just happened, or that the page holds its file. */
function statusText(v: Viewer): string {
  if (v.status !== "" || !v.held || v.following === undefined || "pinned" in v.following) return v.status;
  return zh()
    ? "停在这个文件上，选中别的文件不再换；在上方改选跟随的组即可恢复。"
    : "Staying on this file; choose another binding above to follow the selection again.";
}

function copy(v: Viewer, what: "path" | "value"): void {
  const item = v.selected === undefined ? undefined : v.tree.get(v.selected);
  if (item === undefined) return;
  tessoa.clipboard.writeText(what === "path" ? item.path : JSON.stringify(item.value, null, 2));
  v.status = what === "path" ? "Copied the path" : "Copied the value";
}

tessoa.pages.define<Viewer>("json", {
  open: ({ state }) => {
    const saved = state as Saved | undefined;
    const v: Viewer = {
      file: undefined,
      following: undefined,
      held: false,
      heldOn: undefined,
      name: "",
      tree: new tessoa.pages.TreeModel<Item>(),
      selected: undefined,
      history: new History(),
      current: undefined,
      status: "",
    };
    if (saved === undefined || saved === null || typeof saved !== "object") return v;
    const refs = Array.isArray(saved.history) ? saved.history : typeof saved.file === "number" ? [saved.file] : [];
    v.history.restore(refs);
    if (typeof saved.file === "number" && load(v, saved.file, true)) {
      if (typeof saved.heldOn === "string") {
        v.held = true;
        v.heldOn = saved.heldOn;
        v.following = JSON.parse(saved.heldOn) as tessoa.pagesHost.FollowSource | undefined;
      }
    }
    return v;
  },
  // The history's kept references, and the file shown when it is one of
  // them: tessoa tells the page again what it follows after a restart, and a
  // followed file's reference may no longer work.
  save: (v): Saved | undefined => {
    const history = v.history.saved();
    if (history.length === 0) return undefined;
    const shown = v.history.get(v.current)?.ref;
    const saved: Saved = { history };
    if (shown !== undefined) {
      saved.file = shown;
      if (v.held && v.heldOn !== undefined) saved.heldOn = v.heldOn;
    }
    return saved;
  },
  follow,
  title: (v) => v.name || undefined,
  render: (v) => {
    const item = v.selected === undefined ? undefined : v.tree.get(v.selected);
    const value = item === undefined ? "" : JSON.stringify(item.value, null, 2);
    const entries = v.history.entries;
    return (
      <column pad="m" gap="s" h="fill">
        <row gap="s" align="center">
          <button key="open" icon="search" onClick={() => open(v)}>
            {zh() ? "打开 JSON…" : "Open JSON…"}
          </button>
          <text key="status" tone="dim" grow>
            {statusText(v)}
          </text>
        </row>
        <split key="outer" ratio={0.78} grow>
          <drop-zone
            key="drop"
            accepts="files"
            hint={zh() ? "可把 JSON 文件拖到这里打开。" : "Drop a JSON file here to open it."}
            h="fill"
            onDropFiles={(files) => drop(v, files)}
          >
            <split key="split" ratio={0.55} grow>
              <tree
                key="json"
                count={v.tree.count()}
                empty={emptyText(v)}
                selected={v.selected === undefined ? [] : [v.selected]}
                row={(i) => v.tree.row(i, (_id, it) => ({ cells: [`${it.label}: ${preview(it.value)}`] }))}
                onExpand={(row, open) => expand(v, row, open)}
                onActivate={(row) => {
                  if (isContainer(v.tree.get(row)?.value)) expand(v, row, !v.tree.isOpen(row));
                }}
                onSelect={(ids) => {
                  v.selected = ids[0];
                }}
              />
              <column pad="s" gap="s">
                <text key="path" size="caption" tone="dim">
                  {item === undefined ? (zh() ? "选中一行，查看它的值。" : "Select a row to see its value.") : item.path}
                </text>
                <text key="value" wrap>
                  {value.length > 4000 ? `${value.slice(0, 3999)}…` : value}
                </text>
                <row gap="s">
                  <button key="copy-path" icon="copy" disabled={item === undefined} onClick={() => copy(v, "path")}>
                    {zh() ? "复制路径" : "Copy path"}
                  </button>
                  <button key="copy-value" disabled={item === undefined} onClick={() => copy(v, "value")}>
                    {zh() ? "复制值" : "Copy value"}
                  </button>
                </row>
              </column>
            </split>
          </drop-zone>
          <column gap="xs" h="fill">
            <text key="history-title" size="caption" tone="dim">
              {zh() ? "历史" : "History"}
            </text>
            <list
              key="history"
              grow
              count={entries.length}
              selected={v.current === undefined ? [] : [v.current]}
              empty={zh() ? "看过的文件会列在这里。" : "Files you view are listed here."}
              row={(i) => ({ id: entries[i].id, cells: [entries[i].name] })}
              onSelect={(ids) => reopen(v, ids[0])}
            />
          </column>
        </split>
      </column>
    );
  },
});
