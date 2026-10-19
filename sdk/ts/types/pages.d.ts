// Pages in TypeScript: `tessoa.pages.define`, the elements a page renders,
// their attributes, the rows of lists, tables and trees, and TreeModel.
// Written by hand; the enumerations (Space, Align, TextSize, Fit, Selection,
// Expander and so on), KeyPress and IconName are generated from
// wit/tessoa-plugin.wit into tessoa.d.ts, in the same namespace.
//
// The defaults are the Rust SDK's, so that a page written in either language
// makes the same nodes: a number input takes -1e9 to 1e9 in steps of 1 and
// shows as many decimals as its step has; a slider goes from 0 to 100 in
// steps of 1; a list, a table or a tree selects one row at a time; a column
// is as wide as its content, aligned to the start and not sortable; an image
// is contained in its node.

declare namespace tessoa.pages {
  /**
   * A length: `"auto"`, as large as the content; `"fill"`, as large as the
   * parent allows; or a number of em, units of the current font size.
   */
  type Size = "auto" | "fill" | number;

  interface MenuItem {
    /** What `onMenu` receives. */
    id: string;
    label: string;
    /** Shown, but cannot be chosen. */
    disabled?: boolean;
    /** Drawn in the warning colour, for destructive choices. */
    danger?: boolean;
    /** A separator line follows the item. */
    separatorAfter?: boolean;
  }

  /**
   * What goes between an element's tags: elements, text, numbers and lists
   * of them. null, undefined, true and false show nothing, so that
   * `{done && <text>Done</text>}` works.
   */
  type Child = Element | string | number | bigint | boolean | null | undefined | readonly Child[];

  /** An element, made by JSX or by `h`. Its contents are the engine's. */
  interface Element {
    readonly type: unknown;
    readonly props: unknown;
    readonly key: string | null;
  }

  /** Attributes every node has. */
  interface NodeAttributes {
    /**
     * Names the node in events and in tests, and keeps what the user typed,
     * scrolled and focused across renders. Unique within the whole page: in
     * a list use keys such as `card-12`, not the position. A node with a
     * handler and no key gets a key from its position, such as `#0.2`, which
     * changes when nodes before it come and go; do not start your own keys
     * with `#`.
     */
    key?: string | number;
    /**
     * Take a share of the length the parent has left along its direction.
     * Children that grow get equal shares whatever their content, as with
     * CSS `flex: 1 1 0`, but none gets less than its content needs.
     */
    grow?: boolean;
    /** The width; the default `"auto"` is as wide as the content. */
    w?: Size;
    /** The height; the default `"auto"` is as tall as the content. */
    h?: Size;
    /** Space inside the node, around its children. */
    pad?: Space;
    /** Space between the children. */
    gap?: Space;
    /** Where the children go along the node's direction. */
    justify?: Justify;
    /** The node scrolls when its children do not fit. */
    scroll?: boolean;
    /**
     * A small menu: right-clicking the node opens it, and so does a left
     * click on a button or an icon button that has one, which then does not
     * call its `onClick`. A list's menu acts on its selected rows.
     */
    menu?: MenuItem[];
    /** An item of `menu` was chosen; receives its id. */
    onMenu?: (item: string) => void;
    /**
     * A line shown when the pointer rests on the node, at most 1 KiB. An
     * icon button without one shows its label, an image its `alt`.
     */
    tooltip?: string;
  }

  interface ContainerAttributes extends NodeAttributes {
    /**
     * Where the children go across the node's direction. The default,
     * `"stretch"`, makes them as wide as a column or as tall as a row.
     */
    align?: Align;
    /** Children that do not fit continue on the next line. */
    wrap?: boolean;
    children?: Child;
  }

  interface TextAttributes extends NodeAttributes {
    size?: TextSize;
    tone?: Tone;
    /** Break lines to fit the width instead of cutting the text off. */
    wrap?: boolean;
    /** How the lines are aligned. */
    align?: TextAlign;
    /** The text. */
    children?: Child;
  }

  interface ButtonAttributes extends NodeAttributes {
    kind?: ButtonKind;
    /** An icon from tessoa's icon set, shown with the label. */
    icon?: IconName;
    disabled?: boolean;
    /** A left click. Not called when the button has a `menu`, which the click opens. */
    onClick?: () => void;
    /** The label. */
    children?: Child;
  }

  interface IconButtonAttributes extends NodeAttributes {
    /** An icon from tessoa's icon set. */
    icon: IconName;
    /** The tooltip. */
    label: string;
    kind?: ButtonKind;
    disabled?: boolean;
    /** A left click. Not called when the button has a `menu`, which the click opens. */
    onClick?: () => void;
  }

  interface TextInputAttributes extends NodeAttributes {
    /**
     * The text the page gives the field. The field keeps what the user typed
     * until a render gives it a different value than the render before. To
     * empty the field after a submit, keep its text in the state from
     * `onInput` and set that to "": a page that always gives `value=""`
     * cannot empty it.
     */
    value: string;
    /** Grey text shown while the field is empty. */
    placeholder?: string;
    disabled?: boolean;
    /** Several lines: Enter starts a new line, Ctrl+Enter submits. */
    multiline?: boolean;
    /** The text changed; called on every change. */
    onInput?: (text: string) => void;
    /** Enter, or Ctrl+Enter with several lines; receives the field's text. */
    onSubmit?: (text: string) => void;
  }

  interface CheckboxAttributes extends NodeAttributes {
    checked: boolean;
    /** Receives the new state. */
    onToggle?: (checked: boolean) => void;
    /** The label. */
    children?: Child;
  }

  interface FileRowAttributes extends NodeAttributes {
    /**
     * A file reference issued to the plugin: dropped on the page, or kept
     * with `tessoa.files.keep`.
     */
    file: tessoa.files.FileRef;
    /** Drawn as selected. */
    selected?: boolean;
    /** Whether the user can drag the file out; the default is true. */
    draggable?: boolean;
  }

  interface DropZoneAttributes extends ContainerAttributes {
    /**
     * What the zone takes: files from tessoa or other programs, draggable
     * nodes of the page, or both.
     */
    accepts: Accepts;
    /** Shown in the zone while nothing is dragged over it. */
    hint?: string;
    /** Files were dropped; they arrive as references issued to the plugin. */
    onDropFiles?: (files: tessoa.files.FileRef[]) => void;
    /**
     * A draggable node was dropped: its payload, and the key of the
     * draggable it was dropped in front of, undefined at the end. Only the
     * zone's own children that are draggable and have a key count, and
     * never the one being dragged.
     */
    onDropItem?: (payload: string, before: string | undefined) => void;
  }

  /**
   * Give a draggable a `key`, such as the id of the item it shows:
   * `onDropItem` names the draggable an item was dropped in front of by that
   * key, and never names one without a key.
   */
  interface DraggableAttributes extends ContainerAttributes {
    /** What `onDropItem` of the drop zone receives. */
    payload: string;
  }

  /**
   * An option of a dropdown, a radio group, segmented buttons or tabs. A
   * plain string is an option whose id is its label.
   */
  interface ChoiceOption {
    /** Names the option in `value` and in `onChange`. Unique within the node, not empty. */
    id: string;
    label: string;
    /** An icon from tessoa's icon set before the label. */
    icon?: IconName;
    /** Shown, but cannot be chosen. */
    disabled?: boolean;
  }

  /**
   * A dropdown, a radio group, segmented buttons or tabs. The control shows
   * `value`: render the option `onChange` receives as the new value, or the
   * control goes back to the old one.
   */
  interface ChoiceAttributes extends NodeAttributes {
    /** The id of the chosen option; "" for none. */
    value: string;
    /** At most 256. */
    options: (ChoiceOption | string)[];
    disabled?: boolean;
    /** An option was chosen; receives its id. */
    onChange?: (option: string) => void;
  }

  interface SwitchAttributes extends NodeAttributes {
    checked: boolean;
    /** Receives the new state. */
    onToggle?: (checked: boolean) => void;
    /** The label. */
    children?: Child;
  }

  /**
   * A number field with up and down arrows. It shows `value`: render the
   * number `onChange` receives, or the field goes back to the old one.
   */
  interface NumberInputAttributes extends NodeAttributes {
    value: number;
    /** -1,000,000,000 without it. */
    min?: number;
    /** 1,000,000,000 without it. */
    max?: number;
    /**
     * What the arrows and the arrow keys add or take, and what a typed
     * number is rounded to; 1 without it. Greater than 0.
     */
    step?: number;
    /** The digits after the decimal point; as many as `step` has without it. */
    decimals?: number;
    /** Shown dimmed after the number, such as `px` or `%`. */
    suffix?: string;
    disabled?: boolean;
    /**
     * The new value, within `min` and `max` and on the step: on Enter, when
     * the field loses the focus, and on the arrows; not while the user types.
     */
    onChange?: (value: number) => void;
  }

  /** A slider. It shows `value`: render the number `onChange` receives. */
  interface SliderAttributes extends NodeAttributes {
    value: number;
    /** 0 without it. */
    min?: number;
    /** 100 without it. */
    max?: number;
    /** The values the slider stops at, from `min` on; 1 without it. */
    step?: number;
    /** Upright, with `min` at the bottom. */
    vertical?: boolean;
    disabled?: boolean;
    /** The new value, while the slider is dragged and once more when it is let go. */
    onChange?: (value: number) => void;
  }

  interface ProgressAttributes extends NodeAttributes {
    /**
     * How far, from 0 to 1; undefined when it is not known, which shows a
     * moving bar. A task's `progress` is either.
     */
    value?: number | null;
  }

  interface ImageAttributes extends NodeAttributes {
    /**
     * A PNG, JPEG, GIF, WebP, BMP or SVG file in the plugin's directory,
     * written with `/`, such as `"images/logo.png"`; or the number
     * `tessoa.images.add` returned. Without `w` and `h` the image takes its
     * own size in pixels.
     */
    src: string | number;
    /** How the image fills its node, as CSS `object-fit`; `"contain"` without it. */
    fit?: Fit;
    /** What the image shows, in words; its tooltip when it has none. */
    alt?: string;
  }

  /**
   * Text with bold, italic and code spans and links, written as its
   * children: `<rich-text>Read <b>this</b> or <a href="https://example.com">the
   * site</a> or <a action="more">more</a></rich-text>`.
   */
  interface RichTextAttributes extends NodeAttributes {
    size?: TextSize;
    /** Break lines to fit the width instead of cutting the text off. */
    wrap?: boolean;
    /** How the lines are aligned. */
    align?: TextAlign;
    /** A link with an `action` was clicked; receives the action. */
    onLink?: (action: string) => void;
    /** Text, and `<b>`, `<i>`, `<code>`, `<a>` and `<span>` around text. At most 2048 spans. */
    children?: Child;
  }

  /** `<b>`, `<i>`, `<code>` and `<span>` inside rich text. */
  interface SpanAttributes {
    /** The colour, by meaning. */
    tone?: Tone;
    children?: Child;
  }

  /** `<a>` inside rich text: an `href` or an `action`, not both. */
  interface LinkAttributes extends SpanAttributes {
    /**
     * A web address, `http` or `https`. tessoa shows the user the whole
     * address and opens it in the browser when they agree; the page hears
     * nothing of it.
     */
    href?: string;
    /** An id that `onLink` of the rich text receives. */
    action?: string;
  }

  /** A colour swatch; clicking it opens a colour picker. */
  interface ColorPickerAttributes extends NodeAttributes {
    /** `"#rrggbb"`, or a number 0xrrggbb. */
    value: string | number;
    disabled?: boolean;
    /** The colour the user picked, as `"#rrggbb"`. */
    onChange?: (color: string) => void;
  }

  /** A button showing a date; clicking it opens a calendar. */
  interface DatePickerAttributes extends NodeAttributes {
    /**
     * The date, written `"2026-10-15"`, or a `Date` (its day where the user
     * is); undefined shows an empty button.
     */
    value?: string | Date | null;
    /** Days before this cannot be chosen. */
    min?: string | Date;
    /** Days after this cannot be chosen. */
    max?: string | Date;
    disabled?: boolean;
    /** The day the user chose, as `"2026-10-15"`, or undefined when they cleared it. */
    onChange?: (date: string | undefined) => void;
  }

  /** A column of a table. A list or a tree given columns uses the first. */
  interface Column {
    /** Names the column in `onSort` and `onRowCheck`. Unique within the node, not empty. */
    id: string;
    /** Shown in the header. */
    title: string;
    /** The width the column starts with; the user can drag it, and tessoa remembers it. */
    width?: Size;
    /** How the cells line up. */
    align?: TextAlign;
    /** A click on the header calls `onSort`. */
    sortable?: boolean;
  }

  /** The column the page sorted its rows by, shown as an arrow in the header. */
  interface SortState {
    column: string;
    descending: boolean;
  }

  /**
   * One cell of a row: text or a number; `null` or `undefined` for nothing;
   * `{ text, tone, icon }` for text with a colour or an icon; `{ file }`, a
   * file issued to the plugin, drawn as in tessoa's file list; `{ image }`,
   * a small picture, as for `<image src>`; `{ check }`, a checkbox, which
   * calls `onRowCheck`; `{ progress }`, a bar from 0 to 1.
   */
  type Cell =
    | string
    | number
    | null
    | undefined
    | { text: string; tone?: Tone; icon?: IconName }
    | { file: tessoa.files.FileRef }
    | { image: string | number }
    | { check: boolean }
    | { progress: number };

  /** A button at the end of a row, shown while the pointer is over it. */
  interface RowAction {
    /** What `onRowAction` receives. */
    id: string;
    icon: IconName;
    /** Its tooltip. */
    label: string;
  }

  /**
   * One row of a list, a table or a tree. Rows are data, not nodes: they
   * have no keys and no layout, and all have the same height.
   */
  interface Row {
    /**
     * Names the row in the events; the selection follows it when rows move.
     * At most 128 bytes, unique within the list.
     */
    id: string;
    /** One per column, in order. */
    cells: Cell[];
    /** The second line, dimmed, when the list has `twoLine`. */
    detail?: string;
    /** The level in a tree, 0 at the top. */
    depth?: number;
    /** Whether a tree's row has children, and whether they show. */
    expander?: Expander;
    /** At most 4. */
    actions?: RowAction[];
    /** The colour of the whole row, by meaning. */
    tone?: Tone;
  }

  /**
   * A list, a table or a tree: `count` rows, of which tessoa asks `row` for
   * the ones on screen, so a list can have up to a million. Give it a key.
   * The user selects rows in tessoa at once; the page hears of it in
   * `onSelect`. Beyond two hundred items or so, use one of these instead
   * of a column of nodes.
   */
  interface ListAttributes extends NodeAttributes {
    /** How many rows there are, at most 1,000,000. */
    count: number;
    /**
     * Row `i`, from 0, described from the page's data: the same data gives
     * the same row. Called when tessoa asks, not while the page renders.
     */
    row: (i: number) => Row;
    /**
     * The id of row `i` alone, for selecting rows the user never scrolled
     * to; without it, the id of `row(i)`.
     */
    rowId?: (i: number) => string;
    /** How rows can be selected; `"single"` without it. */
    selection?: Selection;
    /**
     * Selects these rows by id, replacing what the user selected, when they
     * differ from what the previous render gave; without it the selection is
     * the user's. At most 10,000.
     */
    selected?: string[];
    /** Each row has a second, dimmed line: its `detail`. */
    twoLine?: boolean;
    /** Shown when there are no rows. */
    empty?: string;
    /** The selected rows changed; receives their ids, in the order of the rows. */
    onSelect?: (ids: string[]) => void;
    /** A row was opened: double click or Enter. */
    onActivate?: (row: string) => void;
    /** The checkbox of a row's cell was clicked; receives its new state. */
    onRowCheck?: (row: string, column: string, checked: boolean) => void;
    /** A button at the end of a row was clicked; receives its id. */
    onRowAction?: (row: string, action: string) => void;
    /** The first is the one a list or a tree uses. */
    columns?: Column[];
  }

  /** A table: a list with columns under a header that can sort and resize them. */
  interface TableAttributes extends ListAttributes {
    /** At most 32. The cells of a row go with the columns, in order. */
    columns: Column[];
    /** The column the page sorted its rows by; tessoa does not sort. */
    sort?: SortState;
    /**
     * A sortable column's header was clicked: first ascending, then the
     * other way round. Sort the rows and render `sort` to match.
     */
    onSort?: (column: string, descending: boolean) => void;
  }

  /**
   * A tree: a list whose rows have a depth and an expander. The page decides
   * which rows exist; `TreeModel` does that work for it.
   */
  interface TreeAttributes extends ListAttributes {
    /** The expander of a row was clicked: true to show its children. */
    onExpand?: (row: string, open: boolean) => void;
  }

  /** Children in a grid, filled row by row. */
  interface GridAttributes extends ContainerAttributes {
    /**
     * Children per row, 1 to 32. A column is as wide as its widest child;
     * room to spare is shared equally.
     */
    columns: number;
  }

  /** A title that opens and closes the children below it. */
  interface FoldAttributes extends ContainerAttributes {
    title: string;
    /** The children show. Closed, they stay in the tree but are not drawn. */
    open: boolean;
    /** The title was clicked; receives the new state. Render it as `open`. */
    onToggle?: (open: boolean) => void;
  }

  /**
   * Exactly two children side by side, or one above the other, with a bar
   * between them that the user can drag; tessoa remembers where.
   */
  interface SplitAttributes extends NodeAttributes {
    /**
     * The share of the first child, between 0 and 1. tessoa keeps where the
     * user dragged the bar until the page gives another ratio.
     */
    ratio: number;
    /** One above the other instead of side by side. */
    vertical?: boolean;
    /** The bar was let go; receives the share of the first child. */
    onChange?: (ratio: number) => void;
    /** The two sides. */
    children?: Child;
  }

  /**
   * Its children in a panel over the page while `open` is true; `w` is the
   * width of the panel. One popup of a page is open at a time.
   */
  interface PopupAttributes extends ContainerAttributes {
    open: boolean;
    /** A title line with a close button. */
    title?: string;
    /** The key of a node to show the popup below; centred on the page without it. */
    anchor?: string;
    /**
     * Escape, a click outside or the close button: render `open` false to
     * close the popup, which stays open until then.
     */
    onDismiss?: () => void;
  }

  /** The node kinds, `<column>`, `<text>` and so on, with their attributes. */
  interface Elements {
    /** Children top to bottom. */
    column: ContainerAttributes;
    /** Children left to right. */
    row: ContainerAttributes;
    /** A column with a frame and a background. */
    card: ContainerAttributes;
    /** A thin line across the parent. */
    divider: NodeAttributes;
    /** Empty space; with `grow`, pushes its siblings apart. */
    spacer: NodeAttributes;
    text: TextAttributes;
    button: ButtonAttributes;
    /** A button that shows only its icon; the label is its tooltip. */
    "icon-button": IconButtonAttributes;
    /** A text field: one line, or several with `multiline`. */
    "text-input": TextInputAttributes;
    checkbox: CheckboxAttributes;
    /**
     * One file, drawn by tessoa: icon, name, size, and a dimmed look when the
     * file is gone. Double click opens it, the user can drag it out, and its
     * context menu is tessoa's file menu.
     */
    "file-row": FileRowAttributes;
    /** Its children are a target for dragged files or items. */
    "drop-zone": DropZoneAttributes;
    /** Its children can be dragged onto drop zones of the page that take items. */
    draggable: DraggableAttributes;
    /** A button showing the chosen option; clicking it opens the options. */
    dropdown: ChoiceAttributes;
    /** The options one below the other, each with a round mark. */
    "radio-group": ChoiceAttributes;
    /** The options side by side as joined buttons. */
    segmented: ChoiceAttributes;
    /** A row of tabs. Only the tabs: the page shows what belongs to the chosen one below them. */
    tabs: ChoiceAttributes;
    /** An on/off switch. */
    switch: SwitchAttributes;
    "number-input": NumberInputAttributes;
    slider: SliderAttributes;
    /** A progress bar. */
    progress: ProgressAttributes;
    image: ImageAttributes;
    "rich-text": RichTextAttributes;
    "color-picker": ColorPickerAttributes;
    "date-picker": DatePickerAttributes;
    /** Rows in one column. */
    list: ListAttributes;
    table: TableAttributes;
    tree: TreeAttributes;
    grid: GridAttributes;
    fold: FoldAttributes;
    split: SplitAttributes;
    popup: PopupAttributes;
    /** Inside rich text: bold. */
    b: SpanAttributes;
    /** Inside rich text: italic. */
    i: SpanAttributes;
    /** Inside rich text: in a fixed-width font on a faint background. */
    code: SpanAttributes;
    /** Inside rich text: a link. */
    a: LinkAttributes;
    /** Inside rich text: text in a colour. */
    span: SpanAttributes;
  }

  /** What a page's `open` receives. */
  interface OpenInfo {
    /** The page's id, for `tessoa.pagesHost.setTitle`. */
    page: number;
    /**
     * What `save` returned before tessoa restarted or the page was reloaded,
     * parsed from JSON; undefined for a new page. Check its shape: an older
     * version of the plugin may have saved it.
     */
    state: unknown;
    /** What `tessoa.pagesHost.open` passed, parsed from JSON; undefined otherwise. */
    args: unknown;
  }

  /**
   * A kind of page. `S` is the page's state: a plain object that `open`
   * builds and the handlers change in place.
   */
  interface PageDefinition<S> {
    /** Build the state of a page that opens: a plain object, not a promise. */
    open(info: OpenInfo): S;
    /**
     * Describe the whole page. Called after `open`, and after every event and
     * command for every open page. It only describes: no dialogs, no changes
     * to the state or the storage.
     */
    render(state: S): Element;
    /**
     * A value JSON can hold, kept by tessoa and handed back to `open` after a
     * restart. Called with every render. At most 16 KiB; more is dropped and
     * logged.
     */
    save?(state: S): unknown;
    /** The title on the page's tab or sidebar section; without it, the title from plugin.toml. */
    title?(state: S): string | undefined;
    /**
     * Keys the page handles while it has the focus and no text field is
     * focused, written the way they type: "8", "+", "Enter", "Ctrl+z".
     */
    keys?: string[] | ((state: S) => string[]);
    /** A key from `keys` was pressed. */
    onKey?(state: S, key: KeyPress): void;
    /**
     * A copy, a move or a move to the recycle bin that this page started has
     * ended: the transfer id, and one outcome per file, in order.
     */
    onTransferDone?(state: S, transfer: number, outcomes: tessoa.refs.TransferOutcome[]): void;
    /**
     * The page came into view, or went out of view: as a tab, it became the
     * visible tab of its pane or stopped being it; in the sidebar, its
     * section was shown or hidden.
     */
    onActivated?(state: S, visible: boolean): void;
    /** The page closed. */
    close?(state: S): void;
  }

  /**
   * Define a kind of page, under the `id` of its `[[pages]]` entry in
   * plugin.toml. Call it when the script loads, once per kind.
   *
   * Every event and every command is followed by a render of every open
   * page, so a handler only changes the state, in place: what a handler
   * returns is ignored, and there are no hooks such as `useState`. Handlers
   * may be async: `await tessoa.ui.confirm(...)` in a handler shows the
   * question, and the page keeps working while it waits for the answer.
   *
   * Every page of the plugin runs in the same script: a variable outside the
   * state is shared by all of them, and lost when the plugin restarts. Keep
   * what must last in `tessoa.storage`.
   */
  function define<S>(kind: string, page: PageDefinition<S>): void;

  /**
   * Make an element without JSX, for scripts that are not built:
   * `h("button", { key: "add", onClick: add }, "Add")`.
   */
  function h<K extends keyof Elements>(type: K, attributes?: Elements[K] | null, ...children: Child[]): Element;
  function h<P>(type: (props: P) => Element, attributes: P, ...children: Child[]): Element;

  /** One row of a `TreeModel`: the item it shows, and how. */
  interface TreeRow<T> {
    id: string;
    value: T;
    /** The level, 0 at the top. */
    depth: number;
    expander: Expander;
  }

  /** What `describe` of a `TreeModel` makes of an item: a row, whose id, depth and expander the tree sets. */
  type RowDescription = Omit<Row, "id" | "depth" | "expander"> & Partial<Pick<Row, "id">>;

  /**
   * The rows of a tree node, worked out from items. Which rows a tree shows
   * depends on which of its items are open, and tessoa asks for rows by
   * number. A `TreeModel` keeps the items, each with an id unique in the
   * whole tree and a value of the page's choosing, remembers which are
   * open, and answers by number with the item's depth and expander filled
   * in:
   *
   * ```tsx
   * const outline = new tessoa.pages.TreeModel<string>();
   * outline.setRoots([["intro", "Introduction"], ["usage", "Usage"]]);
   * outline.setChildren("usage", [["usage/install", "Install"]]);
   *
   * <tree key="outline" count={outline.count()}
   *       row={(i) => outline.row(i, (id, title) => ({ cells: [title] }))}
   *       onExpand={(row, open) => outline.expand(row, open)} />
   * ```
   *
   * An item with children has an expander; one without has none, unless
   * `setLazy` says its children come later. Opening such an item makes
   * `expand` return true: the page then sets the children, at once or when
   * a task has found them, perhaps with a row such as "Loading…" meanwhile,
   * which is a child like any other. The Rust SDK's `TreeModel` works the
   * same way.
   */
  class TreeModel<T> {
    /**
     * Replace the items at the top, in order, with `[id, value]` pairs. An
     * item that was in the tree before, matched by id, keeps what is below
     * it and stays open or closed, wherever it was; the ones left out go,
     * with everything below them. An id given twice counts once.
     */
    setRoots(items: Iterable<readonly [string, T]>): void;
    /**
     * Replace the children of the item `parent`, the way `setRoots`
     * replaces the items at the top. An item cannot go below itself: the id
     * of `parent` or of an item above it is left out. False, and nothing
     * changes, when the tree has no item `parent`.
     */
    setChildren(parent: string, items: Iterable<readonly [string, T]>): boolean;
    /**
     * The children of `id` come later: it gets an expander, and opening it
     * makes `expand` return true. Children it had go, with everything below
     * them. False when the tree has no item `id`.
     */
    setLazy(id: string): boolean;
    /**
     * Open or close `id`, as `onExpand` asks. True when it opened and its
     * children are still to come. An id the tree does not have is left alone.
     */
    expand(id: string, open: boolean): boolean;
    /** Whether `id` is open. Closed items keep the open state of the items below them. */
    isOpen(id: string): boolean;
    /** The value of the item `id`. */
    get(id: string): T | undefined;
    /** Change the value of `id`; where it is in the tree stays as it is. False when there is no such item. */
    set(id: string, value: T): boolean;
    /** Remove every item. */
    clear(): void;
    /** How many rows the tree shows: the `count` of the tree node. */
    count(): number;
    /** Row `i`, from 0: the item it shows, its depth and its expander. */
    row(i: number): TreeRow<T> | undefined;
    /**
     * Row `i` for the tree node's `row`: `describe` makes it from the item's
     * id and value, and the tree sets its id, depth and expander. tessoa asks
     * only for rows below the count; beyond it this is undefined.
     */
    row(i: number, describe: (id: string, value: T) => RowDescription): Row;
    /** The rows from `start`, at most `count`, made by `describe` as for `row`. */
    rows(start: number, count: number, describe: (id: string, value: T) => RowDescription): Row[];
    /** The ids of the rows from `start`, at most `count`, for the tree node's `rowId`. */
    rowIds(start: number, count: number): string[];
  }
}
