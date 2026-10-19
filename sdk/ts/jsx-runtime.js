// The JSX runtime of @tessoa/plugin-sdk: what `<column gap="s">...</column>`
// in a page's render turns into.
//
// It is not React and has no DOM: an element is a plain object that the
// engine reads when the page renders. TypeScript (`"jsx": "react-jsx"`,
// `"jsxImportSource": "@tessoa/plugin-sdk"`) and esbuild (`--jsx=automatic
// --jsx-import-source=@tessoa/plugin-sdk`) call `jsx` for every element.
//
// An element is { $$tessoa, type, props, key }. `type` is a node kind
// ("column", "text", "button", ...) or the fragment; the engine checks the
// kind and the props, flattens the elements into the page's node list, and
// keeps the handlers. A function component is called right here, with its
// props: components have no state of their own, the page's state is a plain
// object. A key given to a component goes to the element it returns.
//
// The engine's own `tessoa.pages.h` makes the same objects, for scripts
// without a build step. The two symbols are registered globally so that the
// engine and the bundled plugin agree on them.

const ELEMENT = Symbol.for("tessoa.element");

export const Fragment = Symbol.for("tessoa.fragment");

export function jsx(type, props, key) {
  if (typeof type === "function") {
    const made = type(props);
    if (key !== undefined && key !== null && made !== null && typeof made === "object" && made.$$tessoa === ELEMENT && made.key === null) {
      return { ...made, key: String(key) };
    }
    return made;
  }
  return { $$tessoa: ELEMENT, type, props, key: key === undefined || key === null ? null : String(key) };
}

export const jsxs = jsx;
