// Example plugin: a calculator page.
//
// The page shows the number being typed, or the last result, with the
// pending operation above it. Buttons and the keyboard do the same things:
// the page declares the keys it handles, and a click and a key press both
// become one action name. The copy button puts the shown number on the
// clipboard, the one permission the plugin declares. The state survives a
// restart: `save` hands it to tessoa, and `open` gets it back.
//
// The Rust twin in examples/calculator-rs draws the same tree for the same
// input; the tests compare the two node by node. Both round results the same
// way (see formatNumber), so they show the same digits.

type Op = "+" | "-" | "*" | "/";

interface Calculator {
  /** The number shown: what the user is typing, a result, or "Error". */
  display: string;
  /** The left operand of the pending operation. */
  acc: number | null;
  op: Op | null;
  /** The next digit starts a new number instead of extending this one. */
  fresh: boolean;
}

/** Keys the page handles while it has the focus. */
const KEYS = [
  "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "+", "-", "*", "/", "=", "Enter", "Backspace", "Escape",
  "Delete", "Ctrl+c", "Meta+c",
];

/** Longest number the user can type, in characters. */
const MAX_DIGITS = 16;

const SYMBOLS: Record<Op, string> = { "+": "+", "-": "−", "*": "×", "/": "÷" };

function initial(): Calculator {
  return { display: "0", acc: null, op: null, fresh: true };
}

/** A saved state, when it has the shape this version saves. */
function restore(state: unknown): Calculator {
  const s = state as Partial<Calculator> | null | undefined;
  if (
    s &&
    typeof s.display === "string" &&
    (s.acc === null || typeof s.acc === "number") &&
    (s.op === null || (typeof s.op === "string" && s.op in SYMBOLS)) &&
    typeof s.fresh === "boolean"
  ) {
    return { display: s.display, acc: s.acc, op: s.op, fresh: s.fresh };
  }
  return initial();
}

/**
 * A result as the display shows it: rounded to ten decimals, without
 * trailing zeros, "Error" when it is not a number or too large. The rounding
 * is done by hand so that TypeScript and Rust print the same digits: both
 * round the same doubles in the same way, and the fixed-point formatting of
 * the rounded value is then exact in both.
 */
function formatNumber(x: number): string {
  if (!Number.isFinite(x) || Math.abs(x) >= 1e15) return "Error";
  const r = Math.round(Math.abs(x) * 1e10) / 1e10;
  let s = (x < 0 ? -r : r).toFixed(10);
  if (s.includes(".")) s = s.replace(/0+$/, "").replace(/\.$/, "");
  return s === "-0" ? "0" : s;
}

/** The action of a declared key, named like the button that does the same. */
function keyAction(key: string): string | undefined {
  if (/^[0-9]$/.test(key)) return `k${key}`;
  const actions: Record<string, string> = {
    ".": "dot",
    "+": "add",
    "-": "sub",
    "*": "mul",
    "/": "div",
    "=": "eq",
    Enter: "eq",
    Backspace: "back",
    Escape: "clear",
    Delete: "clear",
    "Ctrl+c": "copy",
    "Meta+c": "copy",
  };
  return actions[key];
}

function apply(op: Op, a: number, b: number): number {
  switch (op) {
    case "+":
      return a + b;
    case "-":
      return a - b;
    case "*":
      return a * b;
    case "/":
      return a / b;
  }
}

/** The number shown, or null for "Error". */
function value(c: Calculator): number | null {
  const v = Number(c.display);
  return Number.isNaN(v) ? null : v;
}

function pending(c: Calculator): string {
  return c.acc !== null && c.op !== null ? `${formatNumber(c.acc)} ${SYMBOLS[c.op]}` : "";
}

function digits(c: Calculator): number {
  return c.display.replace(/^-/, "").length;
}

function digit(c: Calculator, d: string): void {
  if (c.fresh || c.display === "0" || c.display === "Error") c.display = d;
  else if (digits(c) < MAX_DIGITS) c.display += d;
  c.fresh = false;
}

function dot(c: Calculator): void {
  if (c.fresh || c.display === "Error") c.display = "0.";
  else if (!c.display.includes(".") && digits(c) < MAX_DIGITS) c.display += ".";
  c.fresh = false;
}

function fail(c: Calculator): void {
  Object.assign(c, initial(), { display: "Error" });
}

function operator(c: Calculator, op: Op): void {
  const v = value(c);
  if (v === null) return;
  if (c.acc !== null && c.op !== null && !c.fresh) {
    const r = apply(c.op, c.acc, v);
    c.display = formatNumber(r);
    if (c.display === "Error") return fail(c);
    c.acc = r;
  } else if (c.acc === null || c.op === null) {
    c.acc = v;
  }
  c.op = op;
  c.fresh = true;
}

function equals(c: Calculator): void {
  const v = value(c);
  if (c.acc === null || c.op === null || v === null) {
    c.fresh = true;
    return;
  }
  c.display = formatNumber(apply(c.op, c.acc, v));
  if (c.display === "Error") return fail(c);
  c.acc = null;
  c.op = null;
  c.fresh = true;
}

function back(c: Calculator): void {
  if (c.fresh || c.display === "Error") return;
  c.display = c.display.slice(0, -1);
  if (c.display === "" || c.display === "-") c.display = "0";
}

/** Do what the button with this key does. */
function press(c: Calculator, action: string): void {
  switch (action) {
    case "clear":
      Object.assign(c, initial());
      return;
    case "back":
      return back(c);
    case "dot":
      return dot(c);
    case "add":
      return operator(c, "+");
    case "sub":
      return operator(c, "-");
    case "mul":
      return operator(c, "*");
    case "div":
      return operator(c, "/");
    case "eq":
      return equals(c);
    case "copy":
      tessoa.clipboard.writeText(c.display);
      return;
  }
  if (/^k[0-9]$/.test(action)) return digit(c, action.slice(1));
  throw new Error(`no action named ${JSON.stringify(action)}`);
}

tessoa.pages.define<Calculator>("calculator", {
  keys: KEYS,
  open: ({ state }) => restore(state),
  save: (c) => c,
  onKey: (c, k) => {
    const action = keyAction(k.key);
    if (action) press(c, action);
  },
  render: (c) => {
    const key = (k: string, label: string, kind?: tessoa.pages.ButtonKind) => (
      <button key={k} grow kind={kind} onClick={() => press(c, k)}>
        {label}
      </button>
    );
    return (
      <column pad="m" gap="s">
        <text key="pending" size="caption" tone="dim" align="end">
          {pending(c)}
        </text>
        <text key="display" size="display" align="end">
          {c.display}
        </text>
        <row gap="s">
          {key("clear", "C")}
          {key("back", "⌫")}
          {key("div", "÷")}
          {key("mul", "×")}
        </row>
        <row gap="s">
          {key("k7", "7")}
          {key("k8", "8")}
          {key("k9", "9")}
          {key("sub", "−")}
        </row>
        <row gap="s">
          {key("k4", "4")}
          {key("k5", "5")}
          {key("k6", "6")}
          {key("add", "+")}
        </row>
        <row gap="s">
          {key("k1", "1")}
          {key("k2", "2")}
          {key("k3", "3")}
          {key("eq", "=", "primary")}
        </row>
        <row gap="s">
          {key("k0", "0")}
          {key("dot", ".")}
          {key("copy", "Copy")}
        </row>
      </column>
    );
  },
});
