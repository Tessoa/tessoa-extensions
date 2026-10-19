// Generated from wit/tessoa-plugin.wit (tessoa:plugin@0.2.0). Do not edit.

/// <reference path="pages.d.ts" />

/**
 * The host interface as a plugin script sees it. Each namespace mirrors one
 * interface of the plugin definition.
 */
declare namespace tessoa {
  /**
   * Plugin log. Lines go to the plugin log panel and the plugin log file.
   * Available without declaring a permission.
   */
  namespace log {
    type Level = "debug" | "info" | "warn" | "error";

    /** Write one line to the plugin log. */
    function log(level: Level, message: string): void;

    /** Write one line at level `debug`. */
    function debug(message: string): void;

    /** Write one line at level `info`. */
    function info(message: string): void;

    /** Write one line at level `warn`. */
    function warn(message: string): void;

    /** Write one line at level `error`. */
    function error(message: string): void;
  }

  /**
   * Notifications and dialogs. The host draws them with its own widgets, so they
   * follow the theme, the UI scale and the language.
   * Available without declaring a permission.
   */
  namespace ui {
    /** Show a notification and return immediately. */
    function notify(title: string, body: string): void;

    /**
     * Ask a yes/no question and resolve to the answer: true when the user
     * pressed the `okLabel` button. In a command the plugin waits for the
     * answer. In a page's handler the question is asked for that page through
     * `tessoa.pagesHost.ask`, and the page keeps working while it waits.
     * The labels default to "OK" and "Cancel".
     */
    function confirm(title: string, body: string, okLabel?: string, cancelLabel?: string): Promise<boolean>;

    /**
     * Like `confirm`, with one line per item shown between the body and the
     * buttons. Use it to preview a batch change before applying it.
     * The labels default to "OK" and "Cancel".
     */
    function confirmList(title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>;

    /**
     * The language tessoa shows its interface in, as a short code such as
     * `en`, `zh`, `ja` or `de`. Use it to pick the plugin's own texts.
     */
    function language(): string;

    /**
     * Offer to open a web address in the user's browser. tessoa shows the
     * whole address and opens it only when the user agrees; the address
     * need not be declared anywhere. Only `http` and `https` addresses are
     * accepted, and only while the plugin handles something the user just
     * did: a command, or a click, a key or a menu choice on a page. At
     * other times, and while the plugin's previous offer is still on
     * screen, the call fails with the reason.
     * Throws an `Error` with the reason when the offer is not made.
     */
    function openUrl(url: string): void;

    /**
     * Open the plugin's page in tessoa's settings, where the settings the
     * manifest declares are. Does nothing for a plugin without settings.
     */
    function openSettings(): void;
  }

  /**
   * The files the user hands to the plugin: the ones selected for the current
   * command, the ones picked in tessoa's file picker, and new files in a folder
   * the user picked.
   */
  namespace files {
    /**
     * Identifies one selected file for the duration of the current command.
     * Handles are issued by the host when the command starts and are invalid
     * after it returns.
     */
    type Handle = number;

    /**
     * Identifies a file the host keeps for the plugin, across commands and
     * across restarts. References come from the user: a file kept with
     * `keep`, a file dropped on a page, a file picked with `pick-files`, or
     * a file the plugin created with `create-file`. The plugin sees the
     * number only.
     */
    type FileRef = number;

    /**
     * Identifies a folder the user picked for the plugin to create files
     * in. It is valid until the plugin stops or tessoa restarts, or until
     * `release-folder`; the plugin sees its name, never its path.
     */
    type Folder = number;

    interface FileInfo {
      handle: Handle;
      /** File name without the directory. Plugins never see the directory. */
      name: string;
      /** Size in bytes. */
      size: number;
      /** Last modification time as Unix epoch milliseconds. */
      modifiedMs: number;
      isDir: boolean;
    }

    /**
     * Why a call was refused:
     * `not-granted`: the handle or reference was not issued to this plugin, or is no longer valid;
     * `permission-denied`: the manifest does not declare the permission this call needs;
     * `io`: the file system or the host reported an error;
     * `invalid`: the request cannot be carried out as made, for example a path with `..` in it, or a call a page cannot make. Carries the reason.
     */
    type ErrorKind = "not-granted" | "permission-denied" | "io" | "invalid";

    /** Thrown by `read`, `rename`, `keep`, `pickFiles`, `pickFolder`, `folderName` and `createFile` when the host refuses the call. */
    interface FilesError extends Error {
      name: "FilesError";
      kind: ErrorKind;
    }

    interface RenameRequest {
      handle: Handle;
      /**
       * New file name without the directory. Names that contain a path
       * separator or `..` are rejected.
       */
      newName: string;
    }

    /**
     * One of:
     * `renamed`: renamed. Carries the final name; the host may have changed it to resolve a conflict.
     * `skipped`: nothing to do, for example because the name did not change. Carries the reason.
     * `failed`: failed. Carries the reason.
     */
    type RenameOutcome = { renamed: string } | { skipped: string } | { failed: string };

    /**
     * One of:
     * `created`: created. Carries a reference to the new file, whose name has a number added when the user chose to keep both.
     * `skipped`: not created: the user chose to skip it because a file of that name exists. Carries the reason.
     */
    type CreateOutcome = { created: FileRef } | { skipped: string };

    /** The files that were selected when the command was triggered. */
    function selected(): FileInfo[];

    /**
     * Read up to `maxLen` bytes starting at `offset`.
     * Requires the `files.read-selected` permission; in `columns.values`,
     * the `files.read-listed` permission.
     * `offset` defaults to 0 and `maxLen` to one megabyte.
     */
    function read(handle: Handle, offset?: number, maxLen?: number): Uint8Array;

    /**
     * Read a whole file.
     * Requires the `files.read-selected` permission.
     */
    function readAll(handle: Handle): Uint8Array;

    /**
     * Ask the host to rename files. One outcome per request, in order; one
     * failure does not stop the others.
     * Requires the `files.rename-selected` permission.
     */
    function rename(items: RenameRequest[]): RenameOutcome[];

    /**
     * Turn a selected file into a reference that stays valid after the
     * command returns, and after tessoa restarts. The host follows the file
     * when tessoa moves or renames it.
     * Requires the `files.keep` permission.
     */
    function keep(handle: Handle): FileRef;

    /**
     * Let the user pick files in tessoa's file picker, and resolve to references
     * to them, which `tessoa.refs.read` reads; empty when the user cancelled.
     * `extensions` limits the files offered, written without the dot, such as
     * `csv`; without it every file shows. With `multiple` the user may pick
     * several. The same in a command, a task and a page's handler: in a handler
     * the page keeps working while the picker is open.
     */
    function pickFiles(options?: PickFilesOptions): Promise<FileRef[]>;

    /** What `pickFiles` shows and takes; every field may be left out. */
    interface PickFilesOptions {
      /** The picker's title, after which tessoa shows the plugin's name. */
      title?: string;
      /** The extensions offered, without the dot, such as `csv`; every file without it. */
      extensions?: string[];
      /** The user may pick several files; one without it. */
      multiple?: boolean;
    }

    /**
     * Let the user pick a folder, and resolve to it; undefined when the user
     * cancelled. The plugin can create files in it with `createFile`, and
     * nothing else, until the plugin stops or tessoa restarts, or until
     * `releaseFolder`; it sees the folder's name (`folderName`), never its path.
     * The same in a command, a task and a page's handler.
     */
    function pickFolder(options?: PickFolderOptions): Promise<Folder | undefined>;

    /** What `pickFolder` shows. */
    interface PickFolderOptions {
      /** The picker's title, after which tessoa shows the plugin's name. */
      title?: string;
    }

    /** The name of a picked folder, without the directories above it. */
    function folderName(folder: Folder): string;

    /** Forget a picked folder; creating files in it fails afterwards. */
    function releaseFolder(folder: Folder): void;

    /**
     * Create a file in a picked folder, and resolve to what happened once it is
     * written. `path` is relative to the folder and written with `/`, such as
     * `report.csv` or `2026/report.csv`; missing folders on the way are created.
     * A path with `..`, a drive, a backslash, a name Windows reserves (such as
     * `con`) or a name that ends in a dot or a space is refused as `invalid`.
     * Text is written as UTF-8; at most 64 MiB. An existing file is never
     * replaced: when the name is taken, tessoa asks the user whether to keep
     * both, with a number added to the new name, or to skip it. The new file is
     * in tessoa's undo and activity log. The same in a command, a task and a
     * page's handler.
     * Requires the `files.create` permission.
     */
    function createFile(folder: Folder, path: string, data: Uint8Array | string): Promise<CreateOutcome>;
  }

  /**
   * Files the host keeps for the plugin. Every function works on references
   * issued to this plugin; any other reference is refused.
   */
  namespace refs {
    /** See `tessoa.files.FileRef`. */
    type FileRef = files.FileRef;

    interface RefInfo {
      ref: FileRef;
      /** File name without the directory. */
      name: string;
      /** Size in bytes; 0 for a folder or a file that is gone. */
      size: number;
      /** Last modification time as Unix epoch milliseconds. */
      modifiedMs: number;
      isDir: boolean;
      /** False when the file is no longer where the host last saw it. */
      exists: boolean;
    }

    /**
     * One of:
     * `done`: copied, moved, or moved to the recycle bin.
     * `skipped`: left alone, for example because the user cancelled. Carries the reason.
     * `failed`: failed. Carries the reason.
     */
    type TransferOutcome = { done: true } | { skipped: string } | { failed: string };

    /**
     * Describe references. References that were never issued to this plugin,
     * or were released, are left out of the result.
     */
    function info(refs: FileRef[]): RefInfo[];

    /**
     * Read up to `maxLen` bytes of a referenced file, starting at `offset`:
     * a file the user dropped on a page or picked, or one the plugin kept or
     * created.
     * Requires the `files.read-selected` permission.
     * `offset` defaults to 0 and `maxLen` to one megabyte.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function read(ref: FileRef, offset?: number, maxLen?: number): Uint8Array;

    /**
     * Read a whole referenced file.
     * Requires the `files.read-selected` permission.
     */
    function readAll(ref: FileRef): Uint8Array;

    /** Forget references. The files themselves are not touched. */
    function release(refs: FileRef[]): void;

    /**
     * Let the user pick a folder and copy the files there, with tessoa's
     * progress, conflict handling and undo. Returns at once with a transfer
     * id; the outcome arrives as a `transfer-done` event of the page that
     * made the call.
     * Requires the `files.transfer` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function copyToPicked(refs: FileRef[]): number;

    /**
     * Like `copy-to-picked`, moving instead of copying. The references
     * follow the files.
     * Requires the `files.transfer` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function moveToPicked(refs: FileRef[]): number;

    /**
     * Open the file the way a double click in tessoa would.
     * Requires the `files.open` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function open(ref: FileRef): void;

    /**
     * Show the file in a new tessoa tab, selected.
     * Requires the `files.open` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function reveal(ref: FileRef): void;

    /**
     * Move the files to the recycle bin (the Trash on macOS). tessoa always
     * asks the user first, in its own dialog that names the plugin; the
     * plugin cannot skip that. Returns at once with a transfer id; the
     * outcome arrives as a `transfer-done` event of the page that made the
     * call.
     * Requires the `files.delete` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function trash(refs: FileRef[]): number;
  }

  /**
   * The plugin's own storage: keys and values that survive restarts, are
   * shared by all of the plugin's pages and commands, and go with tessoa's
   * settings backup. At most 1 MiB in total, keys and values counted; one
   * value at most 256 KiB, one key at most 128 bytes.
   * Available without declaring a permission.
   */
  namespace storage {
    /**
     * Why a call was refused:
     * `quota`: the value, or the storage as a whole, would exceed its limit;
     * `key-too-long`: the key is longer than 128 bytes.
     */
    type ErrorKind = "quota" | "key-too-long";

    /** Thrown by `set` when the host refuses the call. */
    interface StorageError extends Error {
      name: "StorageError";
      kind: ErrorKind;
    }

    /** The value stored under `key`, if any. */
    function get(key: string): Uint8Array | undefined;

    /** Store `value` under `key`, replacing what was there. */
    function set(key: string, value: Uint8Array): void;

    /** Remove `key`. Removing a key that does not exist does nothing. */
    function remove(key: string): void;

    /** Every key, in byte order. */
    function keys(): string[];
  }

  /** The system clipboard. */
  namespace clipboard {
    /**
     * Put text on the clipboard.
     * Requires the `clipboard.write` permission.
     * Throws `tessoa.files.FilesError` when the host refuses the call.
     */
    function writeText(text: string): void;
  }

  /**
   * The plugin's settings: the options `[[settings]]` in `plugin.toml`
   * declares. tessoa shows them in its settings, checks and keeps the values;
   * the plugin only reads them, without a round trip. A setting the user never
   * changed has the default the manifest gives. When the user changes
   * settings, `plugin.on-event` hears of it.
   * Available without declaring a permission.
   */
  namespace settings {
    /**
     * The type of a declared setting: `flag`, `choice`, `number` or `text`;
     * none for an id the manifest does not declare.
     */
    function kind(id: string): string | undefined;

    /**
     * The value of a `flag` setting. False for any other id; an id the
     * manifest does not declare also writes a line to the plugin log.
     */
    function flag(id: string): boolean;

    /**
     * The value of a `number` setting, within its minimum and maximum and
     * on its step. 0 for any other id, as with `flag`.
     */
    function number(id: string): number;

    /**
     * The value of a `text` setting, or the id of the chosen option of a
     * `choice` setting. Empty for any other id, as with `flag`.
     */
    function text(id: string): string;

    /**
     * The value of a setting as its type has it: a `flag` is a boolean, a
     * `number` a number, a `choice` the id of the chosen option and a `text`
     * text. Undefined for an id the manifest does not declare, which also
     * writes a line to the plugin log. Read settings where they are used,
     * in a page's render for example: every page renders again when the
     * user changes them.
     */
    function get(id: string): boolean | number | string | undefined;
  }

  /**
   * Images the plugin makes, for `image` nodes and image cells. The bytes go
   * to tessoa once, when the image is added; pages refer to it by number. The
   * numbers are valid until the plugin stops: a plugin that starts again adds
   * its images again, so a page's state should not keep them.
   * Available without declaring a permission.
   */
  namespace images {
    /**
     * Why a call was refused:
     * `quota`: the plugin's images would take more than 64 MiB once decoded; remove some first;
     * `invalid`: the data is not an image tessoa reads, or it is too large. Carries the reason.
     */
    type ErrorKind = "quota" | "invalid";

    /** Thrown by `add` and `addRgba` when the host refuses the call. */
    interface ImagesError extends Error {
      name: "ImagesError";
      kind: ErrorKind;
    }

    /**
     * Add an image from the bytes of a PNG, JPEG, GIF (its first frame),
     * WebP, BMP or SVG file: at most 8 MiB, and at most 4096 by 4096 pixels.
     * Returns its number.
     * Text is the source of an SVG image.
     */
    function add(data: Uint8Array | string): number;

    /**
     * Add an image from pixels: `width` times `height` times 4 bytes of
     * red, green, blue and alpha, row by row from the top. At most 4096 by
     * 4096 pixels. Returns its number.
     */
    function addRgba(width: number, height: number, data: Uint8Array): number;

    /**
     * Remove an image. Nodes and cells that still name it show an empty
     * frame.
     */
    function remove(id: number): void;
  }

  /**
   * Connections to the websites `plugin.toml` lists under `[permissions] net`,
   * which tessoa shows the user when the plugin is installed; nothing else can
   * be reached. The runner makes the requests: tessoa does not see what is
   * sent or received.
   * Available only to a plugin that lists websites under `[permissions] net`.
   */
  namespace net {
    interface Request {
      /** `GET`, `POST`, `PUT`, `PATCH`, `DELETE` or `HEAD`. */
      method: string;
      /**
       * An `https` address on a listed website, or an `http` address on
       * this computer when the manifest lists it.
       */
      url: string;
      headers: [string, string][];
      /** At most 8 MiB. */
      body: Uint8Array;
      /** 0 for the default of 30 seconds; at most 120 seconds. */
      timeoutMs: number;
    }

    interface Response {
      status: number;
      headers: [string, string][];
      /** At most 32 MiB. */
      body: Uint8Array;
      /** The address that answered, after redirects. */
      url: string;
    }

    /**
     * Why a call was refused:
     * `not-allowed`: the address is not on a listed website, or a redirect left them, or a listed name leads to an address on this computer or the local network. Carries the reason;
     * `timeout`: no answer within the time limit;
     * `too-large`: the answer is larger than 32 MiB, or the request larger than 8 MiB;
     * `failed`: the connection or the request failed. Carries the reason.
     */
    type ErrorKind = "not-allowed" | "timeout" | "too-large" | "failed";

    /** Thrown by `fetch` when the host refuses the call. */
    interface NetError extends Error {
      name: "NetError";
      kind: ErrorKind;
    }

    /**
     * Make a request to a website plugin.toml lists under `[permissions] net`,
     * and resolve to the answer: `fetch(url)` for a GET, or `fetch(url, { method,
     * headers, body, timeoutMs })`, as in a web page. A status such as 404 is an
     * answer, not an error: look at `ok` or `status`. The body is text or bytes;
     * `text()` and `json()` read the answer's. Redirects are followed, at most 5,
     * each to a listed website. No cookies are kept and nothing is cached. At
     * most 8 requests of a plugin are under way at once; more wait their turn.
     * The same in a command, a task, a column and a page's handler.
     */
    function fetch(input: string | FetchRequest, init?: RequestInit): Promise<Response>;

    /** How a request differs from a plain GET; every field may be left out. */
    interface RequestInit {
      /** `GET` unless it says otherwise: `POST`, `PUT`, `PATCH`, `DELETE` or `HEAD`. */
      method?: string;
      /** `{ name: value }` or `[[name, value], ...]`. */
      headers?: Record<string, string> | [string, string][];
      /** Text, sent as UTF-8, or bytes; for JSON, `JSON.stringify(value)`. At most 8 MiB. */
      body?: string | Uint8Array | ArrayBuffer;
      /** How long to wait for the answer; 30 seconds without it, at most 120. */
      timeoutMs?: number;
    }

    /** A request as one object, the address in it. */
    interface FetchRequest extends RequestInit {
      url: string;
    }

    /** What a web page's fetch answers with, besides the fields above. */
    interface Response {
      /** Whether the status is from 200 to 299. */
      readonly ok: boolean;
      /** The body as text, with any bytes that are not UTF-8 replaced. */
      text(): Promise<string>;
      /** The body parsed as JSON. */
      json(): Promise<any>;
      /** The first value of a header, whatever the case of its name. */
      header(name: string): string | undefined;
    }

    /**
     * Abandon a request a page's handler made: pass the promise `fetch`
     * returned. The promise never settles, and the handler that awaits it does
     * not go on. A request of a command, a task or a column has its answer
     * before `fetch` returns.
     */
    function cancel(request: Promise<Response> | number): void;
  }

  /**
   * Long tasks. A task runs in a second instance of the plugin, a worker,
   * while the plugin's pages keep working; the worker's `task-runner.run`
   * receives the input and returns the result. The worker shares the plugin's
   * storage, file references, settings, images and picked folders, but not its
   * variables: whatever it needs goes into the input. tessoa shows every task
   * in its transfer panel, where the user can cancel it.
   * Available without declaring a permission.
   */
  namespace tasks {
    /** How far a task is, as the worker reported it with `task.progress`. */
    interface TaskProgress {
      task: number;
      done: number;
      /** 0 when the total is not known. */
      total: number;
      text: string;
    }

    /**
     * One of:
     * `done`: `run` returned these bytes.
     * `failed`: `run` returned an error, or the worker stopped. Carries the reason.
     * `cancelled`: the user or the plugin cancelled the task.
     */
    type TaskOutcome = { done: Uint8Array } | { failed: string } | { cancelled: true };

    /**
     * Start a task of a kind `define` named: a worker runs it with `input`,
     * which goes as JSON (at most 1 MiB). `title` is shown in tessoa's transfer
     * panel after the plugin's name. Returns the task's handle, which follows its
     * progress and its end; every page renders again whenever the task moves on.
     * At most 4 tasks of a plugin run at once, and 64 more wait their turn. A
     * task a command starts can use the command's files until it ends. Not in a
     * task or a column. Throws an `Error` with the reason when the task cannot
     * start.
     */
    function start(kind: string, title: string, input?: unknown): Task;

    /**
     * Cancel a task, by its handle or its id: `task.cancelled` turns true in
     * its worker, and a worker that has not returned 2 seconds later is stopped.
     * A task still waiting for its turn ends at once. A task that has ended, or
     * that the plugin did not start, is left alone.
     */
    function cancel(task: Task | number): void;

    /**
     * A task the plugin started, as `start` returns it. It follows the task:
     * read `state` and `progress` in a page's render, which runs again
     * whenever the task moves on, and await `done` for the result.
     */
    interface Task<R = any> {
      /** The task's id, as `cancel` knows it. */
      readonly id: number;
      /** The kind `start` named. */
      readonly kind: string;
      /** The title `start` gave. */
      readonly title: string;
      /** `running` until the task ends; then `done`, `failed` or `cancelled`. */
      readonly state: "running" | "done" | "failed" | "cancelled";
      /**
       * How far, from 0 to 1, as the worker last reported; undefined while
       * the total is not known, which `<progress value>` shows as a moving
       * bar; 1 once the task is done.
       */
      readonly progress: number | undefined;
      /** The last report: `done` of `total`, and the line of text. */
      readonly report: { readonly done: number; readonly total: number; readonly text: string };
      /** What the task's run returned, once it is done. */
      readonly result: R | undefined;
      /** Why the task failed, once it has. */
      readonly error: string | undefined;
      /**
       * Resolves to the result. Rejects with a `TaskError` whose kind is
       * `failed` or `cancelled`; a handler that awaits a task the user
       * cancels ends there, without a line in the log. After the await the
       * handler goes on outside the page's call: it can change the state
       * and start tasks, but not wait for the user or the network. A
       * command cannot await it: the task ends after the command returns.
       */
      readonly done: Promise<R>;
      /** Cancel the task, as `tessoa.tasks.cancel` does. */
      cancel(): void;
    }

    /** What a task's run receives besides its input. */
    interface TaskContext {
      /**
       * Report how far the task is: `done` of `total`, and a line of text for
       * tessoa's transfer panel; `total` 0 means the total is not known.
       * Report after every item: a worker that does not call the host for
       * 60 seconds is stopped.
       */
      progress(done: number, total: number, text?: string): void;
      /**
       * Whether the task was cancelled: look between items, clean up and
       * return. A property, not a call.
       */
      readonly cancelled: boolean;
    }

    /** How a task ended without a result, as its `done` rejects. */
    interface TaskError extends Error {
      name: "TaskError";
      kind: "failed" | "cancelled";
    }

    /**
     * Define a kind of task, which `start` names. Call it when the script
     * loads, once per kind. `run` receives the input `start` gave, parsed
     * from JSON, and runs in a worker: another instance of the plugin, which
     * does not see the variables of the pages and the commands. It may be
     * async; in it, `await` on tessoa's dialogs, pickers, new files and
     * `net.fetch` waits for the answer. What it returns, written as JSON (at
     * most 1 MiB), is the task's result; what it throws, the reason it
     * failed. A task that was cancelled ends as cancelled, whatever it
     * returns.
     */
    function define(kind: string, run: (input: any, task: TaskContext) => unknown): void;
  }

  /**
   * What a worker uses while it runs a task. Outside a worker the functions do
   * nothing, and `cancelled` is false.
   * Available without declaring a permission.
   */
  namespace task {
    /**
     * Report progress: `done` of `total`, and a line of text for tessoa's
     * transfer panel; `total` 0 means the total is not known. Report after
     * every item: a worker that does not call the host for 60 seconds is
     * stopped. tessoa shows at most 10 reports a second.
     * `text` defaults to none. In a task's run, `task.progress` does the same.
     */
    function progress(done: number, total: number, text?: string): void;

    /**
     * Whether the task was cancelled. A worker checks between items, cleans
     * up and returns.
     * In a task's run, `task.cancelled` says the same.
     */
    function cancelled(): boolean;
  }

  /**
   * What the host does for a plugin's pages.
   * Available without declaring a permission.
   */
  namespace pagesHost {
    /**
     * Identifies one open page. The host issues it when the page opens; it
     * is unique among all open pages of all plugins.
     */
    type PageId = number;

    /**
     * Ask for the page to be rendered again. The host calls `render` once
     * the current call into the plugin returns, at most 20 times a second
     * per page; several requests in between are one render.
     */
    function invalidate(page: PageId): void;

    /**
     * Open a page of this plugin, declared in the manifest under `kind`,
     * where the first place of its `where` says: a new tab, or a new
     * section of the sidebar. A page declared `single` that is already
     * open in the window is focused instead, and its id returned; `args`
     * then reach no page. From a command, the page opens in the window the
     * command ran in.
     * In TypeScript, `args` is any value JSON can hold; the page's `open`
     * receives it parsed. Throws an `Error` with the host's reason when the
     * page does not open.
     */
    function open(kind: string, args?: unknown): PageId;

    /** Change the page's title, shown on its tab or its sidebar section. */
    function setTitle(page: PageId, title: string): void;

    /**
     * Ask a yes/no question on behalf of a page and resolve to the answer:
     * true for the ok button. The page keeps working while the question waits.
     * With items, the dialog shows one line per item, like `ui.confirmList`.
     * Resolves to false at once when the page is not open. `tessoa.ui.confirm`
     * and `confirmList` in a page's handler ask this way for that page.
     * The labels default to "OK" and "Cancel".
     */
    function ask(page: PageId, title: string, body: string, items: string[], okLabel?: string, cancelLabel?: string): Promise<boolean>;
  }

  /**
   * Values for the columns `[[columns]]` in `plugin.toml` declares, which the
   * user can add to tessoa's file list. tessoa asks only for the files on
   * screen, at most 64 at a time, keeps the answers, and asks again when a
   * file changes. The values are computed in a worker, as for a task. Every
   * plugin exports it; a plugin without columns answers with empty values.
   */
  namespace columns {
    /** See `tessoa.files.Handle`. */
    type Handle = files.Handle;

    /**
     * One file of the file list. The handle is valid during this call
     * only; reading the file needs the `files.read-listed` permission.
     */
    interface ColumnFile {
      handle: Handle;
      /** File name without the directory. */
      name: string;
      /** Size in bytes; 0 for a folder. */
      size: number;
      /** Last modification time as Unix epoch milliseconds. */
      modifiedMs: number;
      isDir: boolean;
    }

    /**
     * One of:
     * `empty`: nothing to show; sorts last.
     * `text`: text; sorts the way tessoa sorts names.
     * `number`: 
     * `size`: a size in bytes.
     * `date`: a time as Unix epoch milliseconds.
     * `sorted-text`: text that sorts by a number, such as `1920×1080` by the pixel count.
     */
    type ColumnValue = { empty: true } | { text: string } | { number: number } | { size: number } | { date: number } | { sortedText: [string, number] };

    /**
     * What a column's value may be: one of `ColumnValue`, or text (sorted as
     * names), a number, a `Date`, or nothing (`null` or `undefined`).
     */
    type ColumnResult = ColumnValue | string | number | Date | null | undefined;

    /**
     * Define how a column `[[columns]]` in plugin.toml declares gets its
     * values. Call it when the script loads, once per column. `value` runs in
     * the column worker, another instance of the plugin, once per file on
     * screen, and may be async; read the file with
     * `tessoa.files.read(file.handle)`, which needs the `files.read-listed`
     * permission. A file whose value throws shows none, and the plugin log
     * says why. A column cannot ask the user anything.
     */
    function define(column: string, value: (file: ColumnFile) => ColumnResult | Promise<ColumnResult>): void;
  }

  /**
   * Pages: whole tabs, or sections of the sidebar, that the plugin describes
   * and tessoa draws. A kind of page is defined with `tessoa.pages.define`;
   * types/pages.d.ts declares it and the elements a page renders.
   */
  namespace pages {
    type Space = "none" | "xs" | "s" | "m" | "l";

    type Align = "start" | "center" | "end" | "stretch";

    type Justify = "start" | "center" | "end" | "between";

    type TextSize = "caption" | "body" | "title" | "display";

    type TextAlign = "start" | "center" | "end";

    type Tone = "normal" | "dim" | "accent" | "danger" | "success";

    type ButtonKind = "normal" | "primary" | "danger" | "quiet";

    /** What a drop zone takes. */
    type Accepts = "files" | "items" | "both";

    /** How an image fills its node, as CSS `object-fit`. */
    type Fit = "contain" | "cover" | "fill" | "none";

    /** How rows of a list, a table or a tree can be selected. */
    type Selection = "none" | "single" | "multi";

    /** Whether a row of a tree has children, and whether they show. */
    type Expander = "none" | "collapsed" | "expanded";

    interface KeyPress {
      /** The key, written as the page declared it. */
      key: string;
      ctrl: boolean;
      shift: boolean;
      alt: boolean;
      /** The Command key on macOS, the Windows key on Windows. */
      meta: boolean;
    }

    /**
     * A name from tessoa's icon set, for `icon` on a button or an icon
     * button. tessoa draws it in the user's theme. Later versions only add
     * names.
     */
    type IconName =
      | "plus"
      | "close"
      | "arrow-left"
      | "arrow-right"
      | "arrow-up"
      | "arrow-down"
      | "chevron-down"
      | "chevron-right"
      | "check"
      | "search"
      | "menu"
      | "more"
      | "reload"
      | "undo"
      | "copy"
      | "cut"
      | "paste"
      | "new-folder"
      | "rename"
      | "delete"
      | "play"
      | "note"
      | "sliders"
      | "eye"
      | "sun"
      | "gear"
      | "image"
      | "keyboard"
      | "monitor"
      | "info"
      | "pin"
      | "question"
      | "tag"
      | "filter"
      | "puzzle";
  }
}

/**
 * `console.log`, `info` and `debug` write to the plugin log at the matching
 * level; `warn` and `error` likewise.
 */
declare var console: {
  log(...args: unknown[]): void;
  info(...args: unknown[]): void;
  debug(...args: unknown[]): void;
  warn(...args: unknown[]): void;
  error(...args: unknown[]): void;
};

/**
 * Decodes UTF-8 bytes into a string, as in browsers and Node.js. Only UTF-8
 * is supported: any other label throws a RangeError. Invalid bytes become
 * U+FFFD unless `fatal` is set, which makes `decode` throw a TypeError. A
 * leading byte order mark is dropped unless `ignoreBOM` is set. Pass
 * `{ stream: true }` while feeding chunks, so that a character split between
 * two chunks is decoded whole.
 */
declare class TextDecoder {
  constructor(label?: "utf-8" | "utf8", options?: { fatal?: boolean; ignoreBOM?: boolean });
  readonly encoding: "utf-8";
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  decode(input?: ArrayBuffer | ArrayBufferView, options?: { stream?: boolean }): string;
}

/** Encodes a string as UTF-8 bytes, as in browsers and Node.js. */
declare class TextEncoder {
  readonly encoding: "utf-8";
  encode(input?: string): Uint8Array;
  /** Writes as much of `source` as fits; `read` counts UTF-16 code units. */
  encodeInto(source: string, destination: Uint8Array): { read: number; written: number };
}
