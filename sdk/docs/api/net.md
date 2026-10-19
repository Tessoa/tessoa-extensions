<!-- Generated from wit/tessoa-plugin.wit by tessoa-witgen. Do not edit. -->

# net

Connections to the websites `plugin.toml` lists under `[permissions] net`,
which tessoa shows the user when the plugin is installed; nothing else can
be reached. The runner makes the requests: tessoa does not see what is
sent or received.
Available only to a plugin that lists websites under `[permissions] net`.

## Functions

### fetch

```
wit  fetch: func(req: request) -> result<response, error>
rust tessoa_plugin_sdk::net::fetch(req: impl Into<Request>) -> Result<Response, Error>   (a Request, or an address for a GET)
ts   tessoa.net.fetch(input: string | FetchRequest, init?: RequestInit): Promise<Response>
```

Make a request and wait for the answer. Redirects are followed, at
most 5, each to a listed website. No cookies are kept and nothing is
cached. At most 8 requests of a plugin are under way at once. A page
cannot wait, and calls `fetch-start` instead.

Permission: the websites listed under `[permissions] net`.

TypeScript also has:

```
```

### fetch-start

```
wit  fetch-start: func(req: request) -> result<u64, error>
rust tessoa_plugin_sdk::net::fetch_start(req: impl Into<Request>) -> Result<u64, Error>   (a Request, or an address for a GET)
ts   none: await tessoa.net.fetch(...) calls it in a page's handler
```

`fetch` for a page: returns at once with a request id, and the
answer arrives as a `fetched` event of the page. Only a page can call
it.

Permission: the websites listed under `[permissions] net`.

### cancel

```
wit  cancel: func(id: u64)
rust tessoa_plugin_sdk::net::cancel(id: u64)
ts   tessoa.net.cancel(request: Promise<Response> | number): void
```

Abandon a request made with `fetch-start`; its `fetched` event does
not arrive.

Permission: the websites listed under `[permissions] net`.

## Types

### request

Record; Rust `Request`, TypeScript `Request`.

| Field | Type | Meaning |
|---|---|---|
| `method` | `string` | `GET`, `POST`, `PUT`, `PATCH`, `DELETE` or `HEAD`. |
| `url` | `string` | An `https` address on a listed website, or an `http` address on this computer when the manifest lists it. |
| `headers` | `list<tuple<string, string>>` |  |
| `body` | `list<u8>` | At most 8 MiB. |
| `timeout-ms` | `u32` | 0 for the default of 30 seconds; at most 120 seconds. |

### response

Record; Rust `Response`, TypeScript `Response`.

| Field | Type | Meaning |
|---|---|---|
| `status` | `u16` |  |
| `headers` | `list<tuple<string, string>>` |  |
| `body` | `list<u8>` | At most 32 MiB. |
| `url` | `string` | The address that answered, after redirects. |

### error

Variant; Rust `Error`, TypeScript `NetError` (thrown) with `kind: ErrorKind`.

| Case | Payload | Meaning |
|---|---|---|
| `not-allowed` | `string` | The address is not on a listed website, or a redirect left them, or a listed name leads to an address on this computer or the local network. Carries the reason. |
| `timeout` | none | No answer within the time limit. |
| `too-large` | none | The answer is larger than 32 MiB, or the request larger than 8 MiB. |
| `failed` | `string` | The connection or the request failed. Carries the reason. |
