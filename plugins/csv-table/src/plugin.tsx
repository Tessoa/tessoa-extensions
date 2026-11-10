// CSV table: a CSV file as a table.
//
// "Open CSV…" lets the user pick a file in tessoa's file picker; the page
// reads it (the picked file is a file reference, read with
// `files.read-selected`), and shows its rows in a table whose header comes
// from the file's first line. A click on a header sorts by that column: the
// page sorts, tessoa only draws the arrow. Columns whose values are all
// numbers sort as numbers and line up at the end. The user selects rows in
// tessoa; Copy, or Ctrl+C, puts them on the clipboard with tabs between the
// cells, ready to paste into a spreadsheet. The page gives the table the
// selection it heard, so that opening another file clears it.
//
// The table has as many rows as the file: tessoa asks `row` for the ones on
// screen, so a file of a hundred thousand lines shows as quickly as one of
// ten. Each row's id is its line in the file, so the selection stays with
// the rows when they are sorted.
//
// The page follows the selection (`follow = true` in plugin.toml), the way
// tessoa's preview tabs do: tessoa draws a chip above it that says what it
// follows, and the user changes it there. When a .csv file is selected in
// the followed group, the page shows it; anything else leaves the page as
// it is.
//
// A file the user hands the page themselves, picked with "Open CSV…",
// dropped on the table from tessoa or another program, or clicked in the
// history, stays: the page stops taking the files selected in the followed
// group, as a pinned preview tab does, until the user chooses another
// binding on the chip. The chip cannot say so (only tessoa changes it), so
// the status line does. A dropped file opens whatever its name, so a .txt
// or a .tsv export opens too.
//
// The column at the right lists the files the page has shown, newest first
// (history.ts says which ones and for how long); clicking one shows it again.
//
// The page saves the history's kept references, the file shown when it is
// one of them, and the sort order. After a restart it reads the file again:
// references stay valid across restarts. A followed file is not saved:
// tessoa says again what the page follows.

import { History } from "./history";

interface Table {
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
  header: string[];
  /** The rows of the file, without the header, each as long as the header. */
  rows: string[][];
  /** Which columns hold numbers only. */
  numeric: boolean[];
  /** The rows in the order they show, as indexes into `rows`. */
  order: number[];
  sort: tessoa.pages.SortState | undefined;
  /** The selected rows, by id, in the order they show. */
  selected: string[];
  history: History;
  /** The history entry of the file shown. */
  current: string | undefined;
  status: string;
}

/** What `save` keeps. Version 1.1 and 1.2 saved `file` and `sort` alone. */
interface Saved {
  /** The kept references of the history, newest first. */
  history?: number[];
  /** The file shown, when it is one of those. */
  file?: number;
  sort?: tessoa.pages.SortState;
  /** The binding the file was chosen on, as JSON, while the page holds it. */
  heldOn?: string;
}

const KEYS = ["Ctrl+c", "Meta+c"];

/** A table has at most 32 columns; the cells after them are left out. */
const MAX_COLUMNS = 32;

/** The character between the cells: `;` when the first line has more of them than commas. */
function delimiter(text: string): string {
  const first = text.slice(0, text.search(/\r?\n|$/));
  const count = (c: string) => first.split(c).length - 1;
  return count(";") > count(",") ? ";" : ",";
}

/**
 * The lines of a CSV file, each split into its cells. A cell in double
 * quotes may hold the delimiter, line breaks and `""` for a quote. Empty
 * lines are left out.
 */
function parseCsv(text: string): string[][] {
  const sep = delimiter(text);
  const lines: string[][] = [];
  let line: string[] = [];
  let cell = "";
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (quoted) {
      if (ch !== '"') cell += ch;
      else if (text[i + 1] === '"') {
        cell += '"';
        i++;
      } else quoted = false;
    } else if (ch === '"' && cell === "") {
      quoted = true;
    } else if (ch === sep) {
      line.push(cell);
      cell = "";
    } else if (ch === "\n" || ch === "\r") {
      if (ch === "\r" && text[i + 1] === "\n") i++;
      line.push(cell);
      lines.push(line);
      line = [];
      cell = "";
    } else {
      cell += ch;
    }
  }
  if (cell !== "" || line.length > 0) {
    line.push(cell);
    lines.push(line);
  }
  return lines.filter((l) => !(l.length === 1 && l[0] === ""));
}

function isNumber(text: string): boolean {
  return text.trim() !== "" && Number.isFinite(Number(text));
}

/** The order of the rows for `sort`: empty cells last, ties in the file's order. */
function sortedOrder(t: Table): number[] {
  const order = t.rows.map((_, i) => i);
  const sort = t.sort;
  if (sort === undefined) return order;
  const c = Number(sort.column.slice(1));
  const key = (i: number) => t.rows[i][c];
  order.sort((a, b) => {
    const [x, y] = [key(a), key(b)];
    if (x === "" || y === "") return x === y ? a - b : x === "" ? 1 : -1;
    let by: number;
    if (t.numeric[c]) by = Number(x) - Number(y);
    else {
      const [lx, ly] = [x.toLowerCase(), y.toLowerCase()];
      by = lx < ly ? -1 : lx > ly ? 1 : 0;
    }
    return (sort.descending ? -by : by) || a - b;
  });
  return order;
}

function plural(n: number, what: string): string {
  return `${n} ${what}${n === 1 ? "" : "s"}`;
}

function zh(): boolean {
  return tessoa.ui.language() === "zh";
}

/** Show a CSV text in the table. */
function show(t: Table, name: string, text: string): void {
  const [first = [], ...rows] = parseCsv(text);
  const header = first.slice(0, MAX_COLUMNS);
  t.name = name;
  t.header = header;
  t.rows = rows.map((r) => header.map((_, c) => r[c] ?? ""));
  t.numeric = header.map((_, c) => t.rows.some((r) => r[c] !== "") && t.rows.every((r) => r[c] === "" || isNumber(r[c])));
  if (t.sort && Number(t.sort.column.slice(1)) >= header.length) t.sort = undefined;
  t.order = sortedOrder(t);
  t.selected = [];
  t.status = plural(t.rows.length, "row");
}

/**
 * Read and show a file. `keep`: the user handed it to the page, which keeps
 * its reference; otherwise it was followed, and the history keeps its text.
 * Returns whether it shows.
 */
function load(t: Table, file: tessoa.files.FileRef, keep: boolean): boolean {
  const [info] = tessoa.refs.info([file]);
  if (info === undefined) return false;
  let text: string;
  try {
    text = new TextDecoder().decode(tessoa.refs.readAll(file));
  } catch (e) {
    t.status = `Cannot read the file: ${(e as Error).message}`;
    return false;
  }
  show(t, info.name, text);
  t.file = file;
  t.current = t.history.note(info, keep ? file : undefined, keep ? undefined : text)?.id;
  return true;
}

/** The user chose what the page shows: it holds it until the chip changes. */
function hold(t: Table): void {
  t.held = true;
  t.heldOn = JSON.stringify(t.following);
}

/** Show a file the user handed the page, and hold it. */
function choose(t: Table, file: tessoa.files.FileRef): void {
  if (load(t, file, true)) hold(t);
  else if (!t.history.entries.some((e) => e.ref === file)) tessoa.refs.release([file]);
}

async function open(t: Table): Promise<void> {
  const [file] = await tessoa.files.pickFiles({ title: "Open a CSV file", extensions: ["csv"] });
  if (file === undefined) return;
  choose(t, file);
}

/** Open the first file dropped on the page; the others are not kept. */
function drop(t: Table, files: tessoa.files.FileRef[]): void {
  const file = tessoa.refs.info(files).find((i) => !i.isDir)?.ref;
  const unused = files.filter((f) => f !== file && !t.history.entries.some((e) => e.ref === f));
  if (unused.length > 0) tessoa.refs.release(unused);
  if (file === undefined) {
    t.status = zh() ? "拖入的不是文件。" : "That is not a file.";
    return;
  }
  choose(t, file);
}

/** Show a file of the history again, and hold it. */
function reopen(t: Table, id: string | undefined): void {
  const entry = t.history.get(id);
  if (entry === undefined || entry.id === t.current) return;
  if (entry.ref !== undefined) {
    if (load(t, entry.ref, true)) hold(t);
  } else if (entry.text !== undefined) {
    show(t, entry.name, entry.text);
    t.file = undefined;
    t.current = entry.id;
    t.history.entries.splice(t.history.entries.indexOf(entry), 1);
    t.history.entries.unshift(entry);
    hold(t);
  }
}

/**
 * Show the .csv file selected in the followed group, unless the page holds
 * a file the user chose and the binding is still the one it was chosen on.
 */
function follow(t: Table, f: tessoa.pagesHost.FollowState): void {
  t.following = f.source;
  if (t.held) {
    if (JSON.stringify(f.source) === t.heldOn) return;
    t.held = false;
  }
  if (f.file === undefined || f.file === t.file) return;
  const [info] = tessoa.refs.info([f.file]);
  if (info === undefined || info.isDir || !/\.csv$/i.test(info.name)) return;
  load(t, f.file, false);
}

/** What the empty table says: where to select a file, in tessoa's language. */
function emptyText(t: Table): string {
  const zh = tessoa.ui.language() === "zh";
  const s = t.following;
  if (s === undefined) return zh ? "打开或拖入一个 CSV 文件，在这里查看它的行。" : "Open or drop a CSV file to see its rows here.";
  if ("pinned" in s) {
    return zh ? "已钉住。打开或拖入一个 CSV 文件，在这里查看它的行。" : "Pinned. Open or drop a CSV file to see its rows here.";
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
  return zh ? `在${where}里选一个 .csv 文件，或打开、拖入一个。` : `Select a .csv file in ${where}, or open or drop one.`;
}

/** The note after the status: that the page holds its file. */
function holdText(t: Table): string {
  if (!t.held || t.following === undefined || "pinned" in t.following) return "";
  return zh()
    ? "停在这个文件上，选中别的文件不再换；在上方改选跟随的组即可恢复。"
    : "Staying on this file; choose another binding above to follow the selection again.";
}

/** Put the selected rows on the clipboard, the header first, cells apart by tabs. */
function copy(t: Table): void {
  if (t.selected.length === 0) return;
  const lines = [t.header, ...t.selected.map((id) => t.rows[Number(id.slice(1))])].map((r) => r.join("\t"));
  tessoa.clipboard.writeText(lines.join("\n"));
  t.status = `Copied ${plural(t.selected.length, "row")}`;
}


tessoa.pages.define<Table>("csv", {
  open: ({ state }) => {
    const saved = state as Saved | undefined;
    const t: Table = {
      file: undefined,
      following: undefined,
      held: false,
      heldOn: undefined,
      name: "",
      header: [],
      rows: [],
      numeric: [],
      order: [],
      sort: undefined,
      selected: [],
      history: new History(),
      current: undefined,
      status: "",
    };
    if (saved === undefined || saved === null || typeof saved !== "object") return t;
    if (saved.sort && typeof saved.sort.column === "string") t.sort = saved.sort;
    const refs = Array.isArray(saved.history) ? saved.history : typeof saved.file === "number" ? [saved.file] : [];
    t.history.restore(refs);
    if (typeof saved.file === "number" && load(t, saved.file, true) && typeof saved.heldOn === "string") {
      t.held = true;
      t.heldOn = saved.heldOn;
      t.following = JSON.parse(saved.heldOn) as tessoa.pagesHost.FollowSource | undefined;
    }
    return t;
  },
  // The history's kept references, the file shown when it is one of them,
  // and the sort order: tessoa tells the page again what it follows after a
  // restart, and a followed file's reference may no longer work.
  save: (t): Saved | undefined => {
    const history = t.history.saved();
    if (history.length === 0) return undefined;
    const shown = t.history.get(t.current)?.ref;
    const saved: Saved = { history, sort: t.sort };
    if (shown !== undefined) {
      saved.file = shown;
      if (t.held && t.heldOn !== undefined) saved.heldOn = t.heldOn;
    }
    return saved;
  },
  follow,
  title: (t) => t.name || undefined,
  keys: KEYS,
  onKey: (t) => copy(t),
  render: (t) => {
    const entries = t.history.entries;
    return (
      <column pad="m" gap="s" h="fill">
        <row gap="s" align="center">
          <button key="open" icon="search" onClick={() => open(t)}>
            {zh() ? "打开 CSV…" : "Open CSV…"}
          </button>
          <button key="copy" icon="copy" disabled={t.selected.length === 0} onClick={() => copy(t)}>
            {zh() ? "复制" : "Copy"}
          </button>
          <text key="status" tone="dim">
            {t.status}
          </text>
          <text key="hold" tone="dim" grow>
            {holdText(t)}
          </text>
        </row>
        <split key="outer" ratio={0.8} grow>
          <drop-zone
            key="drop"
            accepts="files"
            hint={zh() ? "可把 CSV 文件拖到这里打开。" : "Drop a CSV file here to open it."}
            h="fill"
            onDropFiles={(files) => drop(t, files)}
          >
            <table
              key="rows"
              grow
              count={t.order.length}
              columns={t.header.map((title, c) => ({
                id: `c${c}`,
                title,
                sortable: true,
                align: t.numeric[c] ? "end" : "start",
              }))}
              sort={t.sort}
              selection="multi"
              selected={t.selected}
              empty={emptyText(t)}
              row={(k) => {
                const i = t.order[k];
                return { id: `r${i}`, cells: t.rows[i] };
              }}
              onSort={(column, descending) => {
                t.sort = { column, descending };
                t.order = sortedOrder(t);
              }}
              onSelect={(ids) => {
                t.selected = ids;
              }}
            />
          </drop-zone>
          <column gap="xs" h="fill">
            <text key="history-title" size="caption" tone="dim">
              {zh() ? "历史" : "History"}
            </text>
            <list
              key="history"
              grow
              count={entries.length}
              selected={t.current === undefined ? [] : [t.current]}
              empty={zh() ? "看过的文件会列在这里。" : "Files you view are listed here."}
              row={(i) => ({ id: entries[i].id, cells: [entries[i].name] })}
              onSelect={(ids) => reopen(t, ids[0])}
            />
          </column>
        </split>
      </column>
    );
  },
});
