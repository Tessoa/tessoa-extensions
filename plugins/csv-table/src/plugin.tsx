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
// The page follows the selection (`follow = true` in plugin.toml): tessoa
// draws a chip above it that says which tab group it follows, and the user
// changes it there. When a .csv file is selected in that group, the page
// shows it; anything else leaves the page as it is.
//
// The page saves the file reference and the sort order. After a restart it
// reads the file again: references stay valid across restarts. A followed
// file is not saved: tessoa says again what the page follows.

interface Table {
  /** The file shown, and its name. */
  file: tessoa.files.FileRef | undefined;
  /** The file was picked with "Open CSV…", not followed: it is the one to keep. */
  picked: boolean;
  /** What the page follows, as tessoa last said; undefined until it says. */
  following: tessoa.pagesHost.FollowSource | undefined;
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
  status: string;
}

/** What `save` keeps. */
interface Saved {
  file: number;
  sort?: tessoa.pages.SortState;
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

/** Read a picked or a followed file into the table. */
function load(t: Table, file: tessoa.files.FileRef, picked: boolean): void {
  const [info] = tessoa.refs.info([file]);
  let text: string;
  try {
    text = new TextDecoder().decode(tessoa.refs.readAll(file));
  } catch (e) {
    t.status = `Cannot read the file: ${(e as Error).message}`;
    return;
  }
  const [first = [], ...rows] = parseCsv(text);
  const header = first.slice(0, MAX_COLUMNS);
  if (t.file !== undefined && t.file !== file && t.picked) tessoa.refs.release([t.file]);
  t.file = file;
  t.picked = picked;
  t.name = info?.name ?? "";
  t.header = header;
  t.rows = rows.map((r) => header.map((_, c) => r[c] ?? ""));
  t.numeric = header.map((_, c) => t.rows.some((r) => r[c] !== "") && t.rows.every((r) => r[c] === "" || isNumber(r[c])));
  if (t.sort && Number(t.sort.column.slice(1)) >= header.length) t.sort = undefined;
  t.order = sortedOrder(t);
  t.selected = [];
  t.status = plural(t.rows.length, "row");
}

async function open(t: Table): Promise<void> {
  const [file] = await tessoa.files.pickFiles({ title: "Open a CSV file", extensions: ["csv"] });
  if (file === undefined) return;
  load(t, file, true);
}

/** Show the file selected in the followed group, when it is CSV. */
function follow(t: Table, f: tessoa.pagesHost.FollowState): void {
  t.following = f.source;
  if (f.file === undefined || f.file === t.file) return;
  const [info] = tessoa.refs.info([f.file]);
  if (info === undefined || info.isDir || !/\.csv$/i.test(info.name)) return;
  load(t, f.file, false);
}

/** What the empty table says: where to select a file, in tessoa's language. */
function emptyText(t: Table): string {
  const zh = tessoa.ui.language() === "zh";
  const s = t.following;
  if (s === undefined) return zh ? "打开一个 CSV 文件，在这里查看它的行。" : "Open a CSV file to see its rows here.";
  if ("pinned" in s) return zh ? "已钉住。打开一个 CSV 文件，在这里查看它的行。" : "Pinned. Open a CSV file to see its rows here.";
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
  return zh ? `在${where}里选一个 .csv 文件，或打开一个。` : `Select a .csv file in ${where}, or open one.`;
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
    const t: Table = {
      file: undefined,
      picked: false,
      following: undefined,
      name: "",
      header: [],
      rows: [],
      numeric: [],
      order: [],
      sort: undefined,
      selected: [],
      status: "",
    };
    const saved = state as Partial<Saved> | undefined;
    if (saved && typeof saved.file === "number") {
      if (saved.sort && typeof saved.sort.column === "string") t.sort = saved.sort;
      load(t, saved.file, true);
    }
    return t;
  },
  save: (t): Saved | undefined => (t.file === undefined || !t.picked ? undefined : { file: t.file, sort: t.sort }),
  follow,
  title: (t) => t.name || undefined,
  keys: KEYS,
  onKey: (t) => copy(t),
  render: (t) => (
    <column pad="m" gap="s" h="fill">
      <row gap="s" align="center">
        <button key="open" icon="search" onClick={() => open(t)}>
          Open CSV…
        </button>
        <button key="copy" icon="copy" disabled={t.selected.length === 0} onClick={() => copy(t)}>
          Copy
        </button>
        <text key="status" tone="dim" grow>
          {t.status}
        </text>
      </row>
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
    </column>
  ),
});
