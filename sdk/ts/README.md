# @tessoa/plugin-sdk

Type declarations for the host interface a tessoa plugin script sees as the
global `tessoa`, and the JSX runtime for pages.

- `types/tessoa.d.ts` is generated from `wit/tessoa-plugin.wit` in the SDK
  repository; the shapes of the namespaces, records and functions follow
  that file. `types/pages.d.ts` adds the rest of `tessoa.pages`: `define`,
  the elements a page renders and their attributes.
- `jsx-runtime.js` turns the JSX of a page's `render` into the plain objects
  the engine reads. It is not React and needs no DOM.

Add the package as a development dependency and set up `tsconfig.json`:

```json
{
  "compilerOptions": {
    "types": ["@tessoa/plugin-sdk"],
    "jsx": "react-jsx",
    "jsxImportSource": "@tessoa/plugin-sdk"
  }
}
```

and bundle with `esbuild --jsx=automatic --jsx-import-source=@tessoa/plugin-sdk`.
A plugin without pages needs neither JSX setting. The engine component
installs `tessoa` and `console` before the plugin script runs; the JSX
runtime is the only code of this package that ends up in the bundle.

Proprietary. All rights reserved. Same terms as tessoa.
