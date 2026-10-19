/// <reference path="./types/tessoa.d.ts" />

// The JSX runtime of @tessoa/plugin-sdk. The element types and their props
// are declared in types/pages.d.ts, under `tessoa.pages`.

export declare const Fragment: unique symbol;

export declare function jsx(type: unknown, props: unknown, key?: string | number): tessoa.pages.Element;

export declare const jsxs: typeof jsx;

export declare namespace JSX {
  type Element = tessoa.pages.Element;
  interface IntrinsicElements extends tessoa.pages.Elements {}
  interface ElementChildrenAttribute {
    children: {};
  }
  interface IntrinsicAttributes {
    key?: string | number;
  }
}
