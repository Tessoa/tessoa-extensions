// Example plugin: a JSON file as a tree.
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
  file: tessoa.files.FileRef | undefined;
  name: string;
  tree: tessoa.pages.TreeModel<Item>;
  /** The selected row's id. */
  selected: string | undefined;
  status: string;
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

function load(v: Viewer, file: tessoa.files.FileRef): void {
  const [info] = tessoa.refs.info([file]);
  let value: unknown;
  try {
    value = JSON.parse(new TextDecoder().decode(tessoa.refs.readAll(file)));
  } catch (e) {
    v.status = `Not JSON: ${(e as Error).message}`;
    return;
  }
  v.file = file;
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
  if (v.file !== undefined && v.file !== file) tessoa.refs.release([v.file]);
  load(v, file);
}

function copy(v: Viewer, what: "path" | "value"): void {
  const item = v.selected === undefined ? undefined : v.tree.get(v.selected);
  if (item === undefined) return;
  tessoa.clipboard.writeText(what === "path" ? item.path : JSON.stringify(item.value, null, 2));
  v.status = what === "path" ? "Copied the path" : "Copied the value";
}

tessoa.pages.define<Viewer>("json", {
  open: ({ state }) => {
    const v: Viewer = {
      file: undefined,
      name: "",
      tree: new tessoa.pages.TreeModel<Item>(),
      selected: undefined,
      status: "",
    };
    const saved = state as { file?: unknown } | undefined;
    if (saved && typeof saved.file === "number") load(v, saved.file);
    return v;
  },
  save: (v) => (v.file === undefined ? undefined : { file: v.file }),
  title: (v) => v.name || undefined,
  render: (v) => {
    const item = v.selected === undefined ? undefined : v.tree.get(v.selected);
    const value = item === undefined ? "" : JSON.stringify(item.value, null, 2);
    return (
      <column pad="m" gap="s" h="fill">
        <row gap="s" align="center">
          <button key="open" icon="search" onClick={() => open(v)}>
            Open JSON…
          </button>
          <text key="status" tone="dim" grow>
            {v.status}
          </text>
        </row>
        <split key="split" ratio={0.5} grow>
          <tree
            key="json"
            count={v.tree.count()}
            empty="Open a JSON file to see it here."
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
              {item === undefined ? "Select a row to see its value." : item.path}
            </text>
            <text key="value" wrap>
              {value.length > 4000 ? `${value.slice(0, 3999)}…` : value}
            </text>
            <row gap="s">
              <button key="copy-path" icon="copy" disabled={item === undefined} onClick={() => copy(v, "path")}>
                Copy path
              </button>
              <button key="copy-value" disabled={item === undefined} onClick={() => copy(v, "value")}>
                Copy value
              </button>
            </row>
          </column>
        </split>
      </column>
    );
  },
});
