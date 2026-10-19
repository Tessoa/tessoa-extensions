# Connecting to websites

A plugin can make requests to the websites it lists in its manifest, and to
no others. tessoa shows the list to the user when the plugin is installed.
The runner makes the requests: tessoa does not see what is sent or received.

`examples/github-info-ts` is a complete one: a page that asks GitHub's API
about a repository, shows the answer and says what went wrong when there is
none, with scenarios that answer its requests.

## Declare the websites

```toml
[permissions]
net = ["https://api.example.com", "https://*.example.org/v1/"]
```

- Each entry is `https://` and a domain name, optionally with a port and a
  path the addresses must start with: `https://api.example.com/v1` admits
  `/v1` and `/v1/rates`, not `/v10`.
- `*.` in front of the name stands for any one subdomain:
  `https://*.example.org` covers `api.example.org`, not `example.org` and not
  `a.b.example.org`. A `*` anywhere else, or alone, is refused.
- `http://` is only for this computer: `http://localhost`,
  `http://127.0.0.1` or `http://[::1]`, where no port means any port.
- An entry written any other way rejects the manifest, with a message that
  says what is wrong.

A plugin that also reads files (`files.read-selected` or `files.read-listed`)
is shown with one more line when it is installed: it can send what it reads
to the websites listed above it. Ask for both only when the plugin needs
both.

## Make a request

TypeScript, the same in a page's handler, a command, a task and a column:

```ts
const r = await tessoa.net.fetch("https://api.example.com/v1/rates?base=EUR");
if (!r.ok) throw new Error(`the rates answered ${r.status}`);
const rates = await r.json();

await tessoa.net.fetch("https://api.example.com/v1/notes", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ text: "hello" }),
  timeoutMs: 10_000,
});
```

`fetch(url, { method, headers, body, timeoutMs })` looks like a web page's:
the headers are an object or a list of pairs, the body text or bytes (an
object throws; write `JSON.stringify(value)`). The answer has `status`, `ok`
(200 to 299), `headers`, `body` (bytes), `url` (after redirects), `text()`,
`json()` and `header(name)`, whose name is matched in any case. A status such
as 404 is an answer, not an error. In a page's handler the page keeps working
while the request is under way; `tessoa.net.cancel(promise)` abandons it, and
the promise never settles.

Rust, in a command, a task or a column, waits for the answer:

```rust
let r = net::fetch("https://api.example.com/v1/rates?base=EUR").map_err(|e| format!("{e:?}"))?;
let rates = r.text();

// In a page: the answer comes as Event::Fetched with this id.
let request = net::Request::post("https://api.example.com/v1/notes", r#"{"text":"hello"}"#)
    .header("Content-Type", "application/json")
    .timeout(std::time::Duration::from_secs(10));
self.sending = Some(net::fetch_start(request).map_err(|e| format!("{e:?}"))?);
```

A page cannot wait: it calls `net::fetch_start` and the answer arrives as
`Event::Fetched { id, response }` with the id it returned; `net::cancel(id)`
abandons the request, and its event does not arrive. `Request::get`,
`Request::post(url, body)` and `Request::new(method, url)` make requests, with
`.header`, `.body` and `.timeout`; a `&str` or a `String` is a `GET`.
`Response::text()` and `Response::header(name)` read the answer.

A plugin event (Rust `Plugin::on_event`, or a TypeScript handler going on
after `await task.done`) cannot make requests: nobody is waiting for them
there. Make them in a handler or in a task.

## What the runner does

- **Every hop is checked.** Redirects are followed, at most 5, and each must
  go to a listed website. A listed domain name that leads to an address of
  this computer or of the local network is refused, so a website cannot
  redirect the plugin to a router or a printer; list `http://localhost`
  explicitly for a service on this computer.
- **Errors.** `not-allowed` (the address is not on a listed website, a
  redirect left them, or the name leads to the local network), `timeout`,
  `too-large`, and `failed` with the reason. TypeScript throws a `NetError`
  whose `kind` is one of these; Rust returns `Err(net::Error::...)`.
- **Limits.** A request body of at most 8 MiB and an answer of at most
  32 MiB: more is `too-large`, never cut. 30 seconds from the moment the
  request is sent, unless it says otherwise, at most 120. At most 8 requests
  of a plugin are under way at once; the others wait their turn. At most 64
  headers; the ones the runner writes itself, such as `Host` and
  `Content-Length`, are refused, and so is an address with a user name or a
  password in it.
- **Nothing is kept.** No cookies, no cache, no proxy: every request goes
  straight to the website. The user agent is tessoa's, whatever the request
  says. A redirect to another website drops `Authorization` and `Cookie`.
- **A line in the log.** The plugin log gets one line per request and per
  redirect: the method, the website, the status and the size, never the
  query or the content.

## Testing

Scenarios never go to the network. `[[net]]` answers the requests: each
request that passes the runner's checks is answered by the first entry whose
address fits, one hop at a time, and a request no entry answers fails the
step that made it. `expect-request` checks what the plugin sent. See
[testing.md](testing.md#net).

```toml
[[net]]
url = "https://api.example.com/v1/rates?base=EUR"
headers = { "content-type" = "application/json" }
body = '{"USD": 1.08}'

[[steps]]
click = "refresh"
expect-request = { method = "GET", url = "https://api.example.com/v1/rates?base=EUR" }
expect-text = { key = "usd", text = "1.08" }
```

`tessoa-plugin run` makes real requests: it is how to try a command against
the real website.
