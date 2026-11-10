// The files a page has shown, newest first, for the column at its right.
//
// A file the user picked or dropped comes as a reference the page may keep:
// clicking it in the history reads it again, and the references are saved,
// so these entries are there after a restart. A followed file comes as a
// reference that stops working soon after the user selects another one, and
// a page cannot keep it; its entry keeps the text instead, for this run
// only, and for the latest few files of moderate size. A followed file the
// page could not keep the text of is not listed: clicking it would do
// nothing.
//
// The same file seen again moves to the top instead of showing twice: the
// same reference, or the same name, size and modification time.

/** Most entries listed. */
const MAX_ENTRIES = 20;
/** Most followed files whose text is kept. */
const MAX_TEXTS = 10;
/** Longest text kept for a followed file, in characters. */
const MAX_TEXT = 4 * 1024 * 1024;

export interface Entry {
  /** The row id in the list. */
  id: string;
  name: string;
  /** A reference the page keeps: a file the user picked or dropped. */
  ref?: tessoa.files.FileRef;
  /** The text of a followed file, kept for this run only. */
  text?: string;
  /** Name, size and modification time: the same file seen again. */
  key: string;
}

export class History {
  entries: Entry[] = [];
  private next = 0;

  get(id: string | undefined): Entry | undefined {
    return id === undefined ? undefined : this.entries.find((e) => e.id === id);
  }

  /**
   * Note a file the page shows: `ref` when the page keeps its reference,
   * `text` when it does not. Returns its entry, now at the top; undefined
   * when there is nothing to open it again with.
   */
  note(info: tessoa.refs.RefInfo, ref: tessoa.files.FileRef | undefined, text: string | undefined): Entry | undefined {
    const key = `${info.name}\u0000${info.size}\u0000${info.modifiedMs}`;
    const kept = text !== undefined && text.length <= MAX_TEXT ? text : undefined;
    let entry = this.entries.find((e) => (ref !== undefined && e.ref === ref) || e.key === key);
    if (entry === undefined) {
      if (ref === undefined && kept === undefined) return undefined;
      entry = { id: `h${++this.next}`, name: info.name, key };
      this.entries.unshift(entry);
    } else {
      this.entries.splice(this.entries.indexOf(entry), 1);
      this.entries.unshift(entry);
      entry.name = info.name;
      entry.key = key;
    }
    if (ref !== undefined) {
      if (entry.ref !== undefined && entry.ref !== ref) tessoa.refs.release([entry.ref]);
      entry.ref = ref;
      entry.text = undefined;
    } else if (entry.ref === undefined) {
      entry.text = kept;
    }
    this.trim();
    return entry;
  }

  /** Drop the oldest entries past the limits, and the references they kept. */
  private trim(): void {
    let texts = 0;
    this.entries = this.entries.filter((e, i) => {
      if (e.ref === undefined && e.text !== undefined && ++texts > MAX_TEXTS) return false;
      if (i < MAX_ENTRIES) return true;
      if (e.ref !== undefined) tessoa.refs.release([e.ref]);
      return false;
    });
  }

  /** The kept references, newest first, for the page's saved state. */
  saved(): number[] {
    return this.entries.flatMap((e) => (e.ref === undefined ? [] : [e.ref]));
  }

  /** The entries of saved references that still work, in the saved order. */
  restore(refs: number[]): void {
    const infos = tessoa.refs.info(refs.filter((r) => typeof r === "number"));
    for (const r of refs) {
      const info = infos.find((i) => i.ref === r);
      if (info === undefined) continue;
      const key = `${info.name}\u0000${info.size}\u0000${info.modifiedMs}`;
      this.entries.push({ id: `h${++this.next}`, name: info.name, ref: r, key });
    }
    this.trim();
  }
}
