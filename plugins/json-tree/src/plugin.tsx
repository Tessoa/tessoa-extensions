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
// Selecting a row shows its path and its value on the right side of a
// split; the buttons below copy them. The page gives the tree the selection
// it heard, so that opening another file clears it. A double click or Enter on an object
// or an array opens it or closes it, as the expander does.
//
// The page follows the selection (`follow = true` in plugin.toml): tessoa
// draws a chip above it that says which tab group it follows, and the user
// changes it there. When a .json file is selected in that group, the page
// shows it; anything else leaves the page as it is, so that selecting a
// folder or a picture on the way does not empty it. The "Follow selection"
// switch next to "Open JSON…" stops that and starts it again: off, the page
// keeps the file it shows whatever is selected; on again, it shows the
// .json file selected at that moment. The page keeps the switch across
// restarts.
//
// A file dropped on the tree, from tessoa or from another program, opens
// like a picked one, whatever its name: the user chose it, and a file that
// is not JSON says so in the status.
//
// Each row's id is the entry's path, such as `$.plugins[0].id`, which is
// what a scenario names. A row id is at most 128 bytes, so a path longer
// than that gets a number instead.

interface Item {
  path: string;
  /** The key in its object, or the index in its array. */
  label: string;
  value: unknown;
}

interface Viewer {
  /** The page's id, to ask tessoa what it follows when the switch turns on. */
  page: number;
  file: tessoa.files.FileRef | undefined;
  /** The file was picked with "Open JSON…", not followed: it is the one to keep. */
  picked: boolean;
  /** What the page follows, as tessoa last said; undefined until it says. */
  following: tessoa.pagesHost.FollowSource | undefined;
  /** The "Follow selection" switch: off, a selected file does not replace the one shown. */
  followOn: boolean;
  name: string;
  tree: tessoa.pages.TreeModel<Item>;
  /** The selected row's id. */
  selected: string | undefined;
  status: string;
}

/** What `save` keeps. */
interface Saved {
  file?: number;
  follow?: boolean;
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

function load(v: Viewer, file: tessoa.files.FileRef, picked: boolean): void {
  const [info] = tessoa.refs.info([file]);
  let value: unknown;
  try {
    value = JSON.parse(new TextDecoder().decode(tessoa.refs.readAll(file)));
  } catch (e) {
    v.status = `Not JSON: ${(e as Error).message}`;
    return;
  }
  if (v.file !== undefined && v.file !== file && v.picked) tessoa.refs.release([v.file]);
  v.file = file;
  v.picked = picked;
  v.name = info?.name ?? "";
  v.selected = undefined;
  const roots = isContainer(value) ? entries("$", value) : [[idOf("$"), { path: "$", label: "$", value }] as [string, Item]];
  v.tree.clear();
  place(v.tree, undefined, roots);
  v.status = "";
}

async function open(v: Viewer): Promise<void> {
  const [file] = await tessoa.files.pickFiles({ title: "Open a JSON file", extensions: ["json"] });
  if (file === undefined) return;
  load(v, file, true);
}

/** Show the file selected in the followed group, when it is JSON and the switch is on. */
function follow(v: Viewer, f: tessoa.pagesHost.FollowState): void {
  v.following = f.source;
  if (!v.followOn || f.file === undefined || f.file === v.file) return;
  const [info] = tessoa.refs.info([f.file]);
  if (info === undefined || info.isDir || !/\.json$/i.test(info.name)) return;
  load(v, f.file, false);
}

/**
 * Turn following on or off. Turned on, the page shows the .json file
 * selected now; the reference is read at once, before it may stop working.
 */
function setFollow(v: Viewer, on: boolean): void {
  v.followOn = on;
  if (!on) return;
  const f = tessoa.pagesHost.followed(v.page);
  if (f !== undefined) follow(v, f);
}

/** Open the first file dropped on the page; the others are not kept. */
function drop(v: Viewer, files: tessoa.files.FileRef[]): void {
  const infos = tessoa.refs.info(files);
  const file = infos.find((i) => !i.isDir)?.ref;
  const unused = files.filter((f) => f !== file && f !== v.file);
  if (unused.length > 0) tessoa.refs.release(unused);
  if (file === undefined) {
    v.status = zh() ? "拖入的不是文件。" : "That is not a file.";
    return;
  }
  load(v, file, true);
  // Not JSON: nothing changed, and the reference is of no use.
  if (v.file !== file) tessoa.refs.release([file]);
}

function zh(): boolean {
  return tessoa.ui.language() === "zh";
}

/** What the empty tree says: where to select a file, in tessoa's language. */
function emptyText(v: Viewer): string {
  const zh = tessoa.ui.language() === "zh";
  const s = v.following;
  if (!v.followOn) return zh ? "打开或拖入一个 JSON 文件，在这里查看。" : "Open or drop a JSON file to see it here.";
  if (s === undefined) return zh ? "打开一个 JSON 文件，在这里查看。" : "Open a JSON file to see it here.";
  if ("pinned" in s) return zh ? "已钉住。打开一个 JSON 文件，在这里查看。" : "Pinned. Open a JSON file to see it here.";
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
  return zh ? `在${where}里选一个 .json 文件，或打开一个。` : `Select a .json file in ${where}, or open one.`;
}

function copy(v: Viewer, what: "path" | "value"): void {
  const item = v.selected === undefined ? undefined : v.tree.get(v.selected);
  if (item === undefined) return;
  tessoa.clipboard.writeText(what === "path" ? item.path : JSON.stringify(item.value, null, 2));
  v.status = what === "path" ? "Copied the path" : "Copied the value";
}

tessoa.pages.define<Viewer>("json", {
  open: ({ page, state }) => {
    const saved = state as Saved | undefined;
    const v: Viewer = {
      page,
      file: undefined,
      picked: false,
      following: undefined,
      followOn: saved?.follow !== false,
      name: "",
      tree: new tessoa.pages.TreeModel<Item>(),
      selected: undefined,
      status: "",
    };
    if (saved && typeof saved.file === "number") load(v, saved.file, true);
    return v;
  },
  // Only a picked file is kept: tessoa tells the page again what it follows
  // after a restart, and a followed file's reference may no longer work.
  save: (v): Saved | undefined => {
    const saved: Saved = {};
    if (v.file !== undefined && v.picked) saved.file = v.file;
    if (!v.followOn) saved.follow = false;
    return Object.keys(saved).length === 0 ? undefined : saved;
  },
  follow,
  title: (v) => v.name || undefined,
  render: (v) => {
    const item = v.selected === undefined ? undefined : v.tree.get(v.selected);
    const value = item === undefined ? "" : JSON.stringify(item.value, null, 2);
    return (
      <column pad="m" gap="s" h="fill">
        <row gap="s" align="center">
          <button key="open" icon="search" onClick={() => open(v)}>
            {zh() ? "打开 JSON…" : "Open JSON…"}
          </button>
          <switch key="follow" checked={v.followOn} onToggle={(on) => setFollow(v, on)}>
            {zh() ? "跟随选择" : "Follow selection"}
          </switch>
          <text key="status" tone="dim" grow>
            {v.status}
          </text>
        </row>
        <drop-zone
          key="drop"
          accepts="files"
          hint={zh() ? "可把 JSON 文件拖到这里打开。" : "Drop a JSON file here to open it."}
          grow
          onDropFiles={(files) => drop(v, files)}
        >
          <split key="split" ratio={0.5} grow>
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
      </column>
    );
  },
});
