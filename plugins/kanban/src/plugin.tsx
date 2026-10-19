// Example plugin: a task board.
//
// Cards sit in three columns. A new card starts in the first; dragging a
// card onto a column moves it there, before the card it is dropped on. A
// card's menu renames it, in a text field of several lines, or deletes it
// after asking. The board is in the plugin's storage: every board page (one
// per window, `single` in plugin.toml) shows the same cards, a change on one
// shows on the others at once, and the board is still there after a
// restart. What a page is typing or renaming belongs to that page.
//
// Deleting asks with `await tessoa.ui.confirm(...)`. In a page's handler that
// does not hold the page up: the question waits, this page and the others
// keep working, and the handler goes on when the user answers.

interface Card {
  id: string;
  column: string;
  text: string;
}

interface Board {
  /** The number in the next card's id. */
  next: number;
  /** Every card, in the order the columns show them. */
  cards: Card[];
}

interface View {
  /** The new card's text, as typed so far. */
  draft: string;
  /** The card being renamed, and its text as typed so far. */
  editing: string | null;
  editText: string;
}

const COLUMNS = [
  { id: "todo", title: "To do" },
  { id: "doing", title: "Doing" },
  { id: "done", title: "Done" },
];

const MENU = [
  { id: "rename", label: "Rename" },
  { id: "delete", label: "Delete", danger: true },
];

const STORAGE_KEY = "board";

/** The board every page shows. */
let saved: Board | undefined;

function parse(bytes: Uint8Array | undefined): Board {
  if (bytes) {
    try {
      const b = JSON.parse(new TextDecoder().decode(bytes));
      if (b && typeof b.next === "number" && Array.isArray(b.cards)) {
        const cards = b.cards.filter(
          (c: Partial<Card>) =>
            c && typeof c.id === "string" && typeof c.text === "string" && COLUMNS.some((col) => col.id === c.column),
        );
        return { next: b.next, cards };
      }
    } catch {
      // A board this version cannot read starts over.
    }
  }
  return { next: 1, cards: [] };
}

function board(): Board {
  if (saved === undefined) saved = parse(tessoa.storage.get(STORAGE_KEY));
  return saved;
}

function save(): void {
  tessoa.storage.set(STORAGE_KEY, new TextEncoder().encode(JSON.stringify(board())));
}

function firstLine(text: string): string {
  return text.split("\n")[0];
}

function addCard(v: View, text: string): void {
  const t = text.trim();
  if (t === "") return;
  const b = board();
  b.cards.push({ id: `c${b.next++}`, column: COLUMNS[0].id, text: t });
  save();
  v.draft = "";
}

/** Move a card to a column, before the card `before`, or to the column's end. */
function move(id: string, column: string, before: string | undefined): void {
  const b = board();
  const from = b.cards.findIndex((c) => c.id === id);
  if (from < 0) return;
  const [card] = b.cards.splice(from, 1);
  card.column = column;
  const at = b.cards.findIndex((c) => c.id === before && c.column === column);
  if (at < 0) b.cards.push(card);
  else b.cards.splice(at, 0, card);
  save();
}

function rename(v: View, id: string, text: string): void {
  const card = board().cards.find((c) => c.id === id);
  const t = text.trim();
  if (card && t !== "") {
    card.text = t;
    save();
  }
  v.editing = null;
}

async function remove(id: string): Promise<void> {
  const card = board().cards.find((c) => c.id === id);
  if (!card) return;
  const ok = await tessoa.ui.confirm("Delete card", `Delete "${firstLine(card.text)}"?`, "Delete", "Cancel");
  if (!ok) return;
  // Read the board again: other events may have changed it meanwhile.
  const b = board();
  b.cards = b.cards.filter((c) => c.id !== id);
  save();
}

function cardView(v: View, card: Card) {
  return (
    <draggable key={card.id} payload={card.id}>
      <card
        key={`card-${card.id}`}
        pad="s"
        menu={MENU}
        onMenu={(item) => {
          if (item === "rename") {
            v.editing = card.id;
            v.editText = card.text;
          } else {
            return remove(card.id);
          }
        }}
      >
        <text wrap>{card.text}</text>
      </card>
    </draggable>
  );
}

function cardEditor(v: View, card: Card) {
  return (
    <card key={`edit-${card.id}`} pad="s" gap="s">
      <text-input
        key={`text-${card.id}`}
        multiline
        value={card.text}
        onInput={(t) => {
          v.editText = t;
        }}
        onSubmit={(t) => rename(v, card.id, t)}
      />
      <row gap="s" justify="end">
        <button
          key={`cancel-${card.id}`}
          onClick={() => {
            v.editing = null;
          }}
        >
          Cancel
        </button>
        <button key={`save-${card.id}`} kind="primary" onClick={() => rename(v, card.id, v.editText)}>
          Save
        </button>
      </row>
    </card>
  );
}

tessoa.pages.define<View>("board", {
  open: () => ({ draft: "", editing: null, editText: "" }),
  render: (v) => {
    const cards = board().cards;
    return (
      <column pad="m" gap="m">
        <row gap="s" align="center">
          <text-input
            key="new"
            grow
            value={v.draft}
            placeholder="New card"
            onInput={(t) => {
              v.draft = t;
            }}
            onSubmit={(t) => addCard(v, t)}
          />
          <button key="add" kind="primary" disabled={v.draft.trim() === ""} onClick={() => addCard(v, v.draft)}>
            Add
          </button>
        </row>
        <row gap="m" grow>
          {COLUMNS.map((col) => {
            const here = cards.filter((c) => c.column === col.id);
            return (
              <drop-zone
                key={`col-${col.id}`}
                accepts="items"
                grow
                scroll
                pad="s"
                gap="s"
                onDropItem={(payload, before) => move(payload, col.id, before)}
              >
                <text key={`title-${col.id}`} size="title">
                  {`${col.title} (${here.length})`}
                </text>
                {here.map((card) => (card.id === v.editing ? cardEditor(v, card) : cardView(v, card)))}
              </drop-zone>
            );
          })}
        </row>
      </column>
    );
  },
});
