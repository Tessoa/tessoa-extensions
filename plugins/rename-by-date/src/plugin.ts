// Example plugin: rename the selected photos after their capture date.
//
// The command reads the EXIF header of every selected file, builds a name
// like 2026-10-03_1422.jpg, shows the plan in a confirmation dialog and asks
// the host to apply it. Files without a capture date fall back to the
// modification time. Bundled as an IIFE with global name `plugin`, so the
// exported `run` becomes `plugin.run`.

import { captureDate } from "./exif";

const TITLE = "Rename by capture date";

export async function run(commandId: string): Promise<void> {
  switch (commandId) {
    case "rename-by-date":
      return renameByDate();
    default:
      throw new Error(`unknown command ${commandId}`);
  }
}

async function renameByDate(): Promise<void> {
  const selected = tessoa.files.selected();
  if (selected.length === 0) {
    tessoa.ui.notify(TITLE, "No files selected.");
    return;
  }
  tessoa.log.info(`${selected.length} files selected`);

  const plan: { file: tessoa.files.FileInfo; newName: string }[] = [];
  const used = new Set<string>();
  for (const file of selected) {
    if (file.isDir) {
      tessoa.log.info(`skipping folder ${file.name}`);
      continue;
    }
    // EXIF lives in the APP1 segment near the start of the file.
    let head: Uint8Array;
    try {
      head = tessoa.files.read(file.handle, 0, 256 * 1024);
    } catch (e) {
      tessoa.log.warn(`cannot read ${file.name}: ${(e as Error).message}`);
      continue;
    }
    const date = captureDate(head);
    let stem: string;
    if (date !== null) {
      stem = stemFromExif(date);
    } else {
      tessoa.log.info(`${file.name} has no capture date; using the modification time`);
      stem = stemFromUnixMs(file.modifiedMs);
    }
    const dot = file.name.lastIndexOf(".");
    const ext = dot > 0 ? file.name.slice(dot + 1).toLowerCase() : "";
    let candidate = withExt(stem, ext);
    for (let n = 2; used.has(candidate); n++) {
      candidate = withExt(`${stem}_${n}`, ext);
    }
    used.add(candidate);
    plan.push({ file, newName: candidate });
  }

  if (plan.length === 0) {
    tessoa.ui.notify(TITLE, "Nothing to rename.");
    return;
  }

  const lines = plan.map((p) => `${p.file.name}  ->  ${p.newName}`);
  const body = `${plan.length} files will be renamed:`;
  if (!(await tessoa.ui.confirmList(TITLE, body, lines, "Rename", "Cancel"))) {
    tessoa.log.info("cancelled by the user");
    return;
  }

  let outcomes: tessoa.files.RenameOutcome[];
  try {
    outcomes = tessoa.files.rename(plan.map((p) => ({ handle: p.file.handle, newName: p.newName })));
  } catch (e) {
    throw new Error(`the rename request was refused: ${(e as Error).message}`);
  }

  let renamed = 0;
  outcomes.forEach((outcome, i) => {
    const name = plan[i].file.name;
    if ("renamed" in outcome) {
      renamed++;
      tessoa.log.info(`${name} -> ${outcome.renamed}`);
    } else if ("skipped" in outcome) {
      tessoa.log.info(`skipped ${name}: ${outcome.skipped}`);
    } else {
      tessoa.log.error(`failed ${name}: ${outcome.failed}`);
    }
  });
  tessoa.ui.notify(TITLE, `Renamed ${renamed} of ${plan.length} files.`);
}

function withExt(stem: string, ext: string): string {
  return ext ? `${stem}.${ext}` : stem;
}

/** "2026:10:03 14:22:31" becomes "2026-10-03_1422". */
export function stemFromExif(date: string): string {
  if (date.length >= 16) {
    return `${date.slice(0, 4)}-${date.slice(5, 7)}-${date.slice(8, 10)}_${date.slice(11, 13)}${date.slice(14, 16)}`;
  }
  return date.replace(/[: ]/g, "-");
}

/** Unix milliseconds become "YYYY-MM-DD_HHMM" in UTC. */
export function stemFromUnixMs(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}_${pad(d.getUTCHours())}${pad(d.getUTCMinutes())}`;
}
