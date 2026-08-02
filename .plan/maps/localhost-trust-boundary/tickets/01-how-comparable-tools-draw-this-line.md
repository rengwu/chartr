---
type: research
claimed_by: s3d41a500e84d
claimed_at: 2026-08-02T18:09:05Z
---

# How comparable local-server tools draw this line

## Question

chartr is one of a large family: a program that binds a local HTTP port, serves a
browser UI, and exposes something dangerous behind it. Several of that family have
been through exactly the incident chartr just avoided, and their answers — including
the ones they tried and abandoned — are the cheapest evidence available for tickets
02 through 06.

Find out what each of these actually does *today*, and where the history is visible,
what they did before and why they changed:

- **Jupyter Notebook / Lab.** The reference point for "local server, arbitrary code
  execution, browser UI." What it does about origin and Host checking, and its token
  model — where the token lives, how the browser gets it, what the user experience
  costs. Jupyter has been through public vulnerabilities here; find what forced each
  change.
- **Vite** — directly relevant, because the dev proxy is the stated reason chartr
  disabled its check. Vite added host checking after a rebinding vulnerability of
  its own; find how `server.host`, `server.allowedHosts` and its origin handling
  landed, and what the default is now. This is the closest thing to a
  ready-made answer for ticket 01 of the fix map's dev-origin problem.
- **code-server / openvscode-server.** Remote-capable by design, so its posture is
  necessarily stronger — but note *what* it requires and at what point (password,
  proxy, tunnel), because it shows what the price of authentication actually is.
- **ttyd and gotty.** The closest functional analogues: a terminal over a
  websocket. What they do about origin, what their defaults are, and whether their
  documentation warns about exposure. gotty in particular has a documented history
  worth reading.
- **Ollama.** Ships origin checking for a local API with no auth, which is close to
  the position chartr may land on; find what its allowed-origin list contains and
  why.

For each: the default posture, what is enforced versus documented, how development
origins are admitted without weakening the shipped default, what happens on a
non-loopback bind, and — most useful of all — any public incident that moved them.

**What this ticket is not.** It is not a recommendation and must not contain one.
Tickets 02–04 do the deciding; this one gives them something to decide against.
Where a tool's answer looks wrong for chartr, record it anyway with the reason it
does not transfer, rather than filtering the survey down to the appealing options.

**Watch for the shape that matters most.** Nearly every tool here has to solve
"admit the dev origin without shipping a hole" — the exact tension the comment at
`control.go:23-25` resolved the wrong way. If a pattern recurs across three or more
of them, that is the finding, and it should be called out at the top of the answer
rather than buried per-tool.

Done when: each tool above has its current posture recorded with a citation, the
incidents that shaped them are named where they exist, the recurring dev-origin
pattern is called out explicitly, and nothing in the answer recommends a course for
chartr.

## Answer

Research snapshot: 2026-08-03. “Default” below means the upstream command or image
named explicitly; several of these projects deliberately give their native binary
and container image different defaults.

### The recurring development-origin pattern

Five of the surveyed tools preserve the normal gate and add the exceptional caller,
rather than turning the gate off globally:

- Jupyter keeps same-origin checks and admits a configured exact origin/regex, or a
  request carrying its token.
- Vite has separate additive controls for request `Host` (`allowedHosts`) and browser
  origin (`cors.origin`); its backend-integration instructions say to add the backend
  origin specifically.
- code-server compares WebSocket `Origin` with the effective `Host`, with repeatable
  `--trusted-origin` exceptions for reverse proxies whose origin it trusts but does
  not control.
- GoTTY keeps the WebSocket library's same-origin default and offers `--ws-origin`
  as a regex exception.
- Ollama keeps a built-in list of loopback/native-shell origins and appends
  `OLLAMA_ORIGINS` entries.

That is an observed common shape, not a claim that the controls are equivalent.
In particular, an Origin/CORS rule constrains browser callers; it is not
authentication for a non-browser client. ttyd is the counterexample: its Origin
check is an off-by-default boolean, so enabling a proxy/dev caller and retaining the
check are not independently configurable.

### Jupyter Notebook / JupyterLab (Jupyter Server)

**Current default.** Jupyter Server listens on `localhost`, generates a fresh random
authentication token when no password is configured, and treats access as arbitrary
code execution. The token may arrive in an `Authorization: token ...` header, a
`?token=...` URL, or the login form. The first successful browser visit establishes
a cookie, so the recurring UX cost is normally one automatic bootstrap rather than a
login on every launch. The server logs copyable token URLs; its preferred browser
launch writes a permission-protected local HTML redirect file so the token is not
exposed in the browser process command line. These behaviors and the opt-out warning
are in Jupyter's [current security documentation](https://jupyter-server.readthedocs.io/en/latest/operators/security.html).

**Origin and Host enforcement.** HTTP API and WebSocket handlers use the same
origin decision: same `Origin`/`Host` is accepted; `ServerApp.allow_origin` or
`allow_origin_pat` can admit an explicit cross-origin UI; a token-authenticated
request may skip the Origin/XSRF check. With the default loopback bind, an unknown
`Host` is rejected as DNS-rebinding protection; `local_hostnames` is the additive
hostname escape hatch. The implementation is visible in
[`JupyterHandler.check_origin` and `check_host`](https://github.com/jupyter-server/jupyter_server/blob/main/jupyter_server/base/handlers.py).

On a non-loopback or wildcard bind, `allow_remote_access` dynamically defaults to
true, disabling the local-Host restriction, while token authentication remains. The
dynamic default is explicit in
[`ServerApp._default_allow_remote`](https://github.com/jupyter-server/jupyter_server/blob/main/jupyter_server/serverapp.py#L1369),
and the operator guide separately says a public server should use password plus
HTTPS ([public-server guide](https://jupyter-server.readthedocs.io/en/stable/operators/public-server.html)).
Thus the wide-bind transition is enforced for authentication but documented, rather
than enforced, for TLS/password hardening.

**History.** Notebook 4.3 made token authentication on by default and arranged for
automatic browser launch to consume it. The same release line shortly fixed
CVE-2016-9971, a CSRF that could create files and start kernels in some browsers;
the [4.x changelog](https://jupyter-notebook.readthedocs.io/en/4.x/changelog.html#id4)
records both the new token UX and that incident. The Host check arrived in 2018 as a
defense-in-depth measure, not in response to a known exploit: the maintainers stated
that token auth should already stop rebinding but wanted protection for flaws and
for users who disabled auth
([PR #3714](https://github.com/jupyter/notebook/pull/3714)). The redirect-file launch
then removed the token from browser command-line arguments; current source still
describes the multi-user token-theft concern
([`use_redirect_file`](https://github.com/jupyter-server/jupyter_server/blob/main/jupyter_server/serverapp.py#L1410)).

### Vite

**Current default.** Vite's dev server binds `localhost`. Its current
[`server.allowedHosts`](https://vite.dev/config/server-options.html#server-allowedhosts)
default admits `localhost`, subdomains of `.localhost`, and literal IP addresses;
other names must be added explicitly. `true` disables the check and is documented as
DNS-rebinding exposure. HTTPS skips this Host check. Browser reads have a separate
[`server.cors`](https://vite.dev/config/server-options.html#server-cors) default that
admits only HTTP(S) loopback origins (`localhost`, `127.0.0.1`, and `::1`, any port),
not `*`.

`server.host=true`/`0.0.0.0` makes the listener reachable on LAN/public interfaces
but does not itself set `allowedHosts=true` or CORS to `*`; all literal IP Hosts still
pass, while arbitrary domain Hosts and non-loopback browser origins do not. Vite's
development/proxy escape hatches are additive: configure the precise
`allowedHosts` entry, add the backend UI to `cors.origin`, or inject one deployment
hostname with `__VITE_ADDITIONAL_SERVER_ALLOWED_HOSTS`. `server.origin` is different:
it changes generated asset URLs and is not an access-control allowlist.

**History.** Before the January 2025 fixes, Vite used `Access-Control-Allow-Origin:
*`, did not validate WebSocket Origin, and did not validate HTTP Host. A malicious
web page could read a loopback-only Vite server, hijack HMR WebSockets, or use DNS
rebinding. The project advisory
[GHSA-vg6x-rcgg-rjx6 / CVE-2025-24010](https://github.com/vitejs/vite/security/advisories/GHSA-vg6x-rcgg-rjx6)
names all three causes and says the vulnerability affected even localhost-only users;
the fix introduced the current narrow CORS default, WebSocket token validation, and
`allowedHosts`. In April 2026,
[GHSA-p9ff-h696-f583 / CVE-2026-39363](https://github.com/vitejs/vite/security/advisories/GHSA-p9ff-h696-f583)
found a different WebSocket path that could read arbitrary files when Vite was
explicitly network-exposed; patched releases apply the HTTP filesystem boundary to
that path too.

### code-server and OpenVSCode Server

These projects share an editor-shaped attack surface but do not share a default
trust model.

**code-server.** Its generated config binds `127.0.0.1:8080`, enables password auth,
creates a random password in the config file, and serves plain HTTP. Login produces a
browser cookie; attempts are rate-limited. Changing `bind-addr` does not implicitly
drop auth. The official guide says never expose it without both authentication and
encryption, recommends an SSH tunnel (where disabling code-server's own password is
shown only after the tunnel becomes the gate), and documents reverse-proxy/external
identity alternatives
([setup guide](https://github.com/coder/code-server/blob/main/docs/guide.md),
[default config](https://github.com/coder/code-server/blob/main/docs/FAQ.md#how-does-the-config-file-work)).

Since 4.10.1, authenticated WebSockets also enforce Origin against the effective
Host to cover browsers without useful SameSite-cookie protection and related
subdomains. Proxies must forward Host; `--trusted-origin` (added in 4.15.0) is the
narrow exception for a trusted proxy origin. The sequence is recorded in the
[code-server changelog](https://github.com/coder/code-server/blob/main/CHANGELOG.md#4101---2023-03-04).
An earlier concrete failure also shaped the boundary: 4.5.2 fixed port-proxy routes
that omitted authentication, making local HTTP services reachable through an
otherwise password-protected code-server. I found no CVE assigned to either change.

**OpenVSCode Server.** The native server defaults to `localhost` and, absent an
override, generates a UUID connection token. A `?tkn=` URL stores the token in a
SameSite=Lax cookie and redirects to a clean URL; all HTTP requests and the editor
WebSocket are token-gated. The token can instead come from a file. This is enforced
in [`serverConnectionToken.ts`](https://github.com/gitpod-io/openvscode-server/blob/main/src/vs/server/node/serverConnectionToken.ts).
`--without-connection-token` is an explicit bypass whose CLI help says to use it only
when another layer secures the connection
([server options](https://github.com/gitpod-io/openvscode-server/blob/main/src/vs/server/node/serverEnvironmentService.ts)).

The important packaging exception is that the official Docker image binds
`0.0.0.0:3000` **and passes `--without-connection-token` by default**
([Dockerfile entrypoint](https://github.com/gitpod-io/openvscode-releases/blob/main/Dockerfile)).
The project README documents how to replace that entrypoint with
`--connection-token` or a token file
([security section](https://github.com/gitpod-io/openvscode-server#securing-access-to-your-ide)).
I found no comparable configurable Host/Origin gate and no public incident that
changed this posture. The native binary's token survives a non-loopback `--host`;
the official container's no-token/wide-bind combination relies on deployment
isolation or an external gate, but that reliance is documented rather than enforced.

### ttyd and GoTTY

**ttyd.** Current ttyd listens on port 7681 with no interface specified (the
libwebsockets all-interface behavior), no authentication, and a read-only terminal.
Basic auth (`--credential`) and reverse-proxy auth (`--auth-header`) are optional.
Same-origin WebSocket validation is also optional: `--check-origin` turns it on,
while the default accepts cross-origin sockets. Its CLI lists all of these as
independent switches
([README options](https://github.com/tsl0922/ttyd#command-line-options)); current
source confirms that auth is checked on the WebSocket upgrade but Host/Origin is
checked only when that flag is set
([`protocol.c`](https://github.com/tsl0922/ttyd/blob/main/src/protocol.c)). There is
no additive allowed-origin list, no bind-sensitive escalation, and no exposure
warning in the main README. Read-only limits keystrokes but does not make terminal
output private.

This code reflects a public incident. In 2017, ttyd <=1.3.0 authenticated the HTTP
page but not the WebSocket protocol path; an unauthenticated remote client could
bypass Basic auth and execute shell commands. The vendor patched it the day it was
reported. Fox-IT's
[technical advisory](https://www.fox-it.com/nl-en/research/technical-advisory-remote-shell-commands-execution-in-ttyd/)
names the affected versions and fix, and the current upgrade path still performs
the Basic-auth check before accepting the WebSocket.

**GoTTY.** The last upstream release is 1.0.1 (2017), so “current” here is old code,
not a recently maintained posture. It binds `0.0.0.0:8080`, has no auth or TLS, and
is read-only by default. `--permit-write` is explicitly marked dangerous; optional
controls are Basic auth, a random URL, TLS, or TLS client certificates. The README
warns that Basic credentials are plaintext without TLS and strongly warns before
enabling writes
([GoTTY security options](https://github.com/yudai/gotty#security-options)).

Unlike ttyd, GoTTY rejects cross-origin WebSockets by default through Gorilla
WebSocket's same-Host behavior. `--ws-origin` replaces that with an operator-supplied
regular expression; the implementation and default are visible in
[`server.go`](https://github.com/yudai/gotty/blob/master/server/server.go#L55) and
[`options.go`](https://github.com/yudai/gotty/blob/master/server/options.go#L29).
That exception was added in 2017 specifically to allow selected cross-origin
endpoints
([commit 6765efb](https://github.com/yudai/gotty/commit/6765efbd6148008fb37306dc0dcb30fd7b405811)).
I found no public vulnerability report that forced the change. GoTTY performs no
Host allowlisting and makes no stronger transition on a non-loopback bind because
the default bind is already all interfaces.

### Ollama

**Current default.** Ollama binds `127.0.0.1:11434` and its local API has no client
authentication gate. It does enforce two browser-facing checks. First, on a loopback
listener it rejects an unrecognised HTTP Host; local/loopback/private/interface IPs,
the machine hostname, and `.localhost`, `.local`, and `.internal` names pass. The
Host middleware is skipped when Ollama is bound non-loopback
([current route source](https://github.com/ollama/ollama/blob/main/server/routes.go#L1785)).
Second, CORS allows HTTP(S) origins on `localhost`, `127.0.0.1`, and `0.0.0.0` at any
port, plus native-shell origins `app://*`, `file://*`, `tauri://*`,
`vscode-webview://*`, and `vscode-file://*`. `OLLAMA_ORIGINS` appends entries rather
than replacing those defaults
([`AllowedOrigins`](https://github.com/ollama/ollama/blob/main/envconfig/config.go#L80)).

`OLLAMA_HOST=0.0.0.0` is the documented network-exposure switch. It does not add
authentication or a warning/refusal; Host protection is deliberately bypassed on
that listener, while CORS continues to constrain browser reads to the origin list.
The FAQ documents the bind and additional-origin variables and shows reverse
proxy/tunnel setups
([network and origins FAQ](https://github.com/ollama/ollama/blob/main/docs/faq.mdx#how-can-i-expose-ollama-on-my-network)).

**History and rationale.** The original origin discussion explicitly rejected
`Access-Control-Allow-Origin: *` because any visited website could call the local API,
and proposed operator approval for extra origins
([issue #300](https://github.com/ollama/ollama/issues/300)); `OLLAMA_ORIGINS` became
the non-interactive form of that approval. Later built-in entries track actual native
clients (Tauri, VS Code webviews/files), while browser extensions remain explicit in
the FAQ. I found no published incident tied to the origin list or Host middleware.
Open feature requests for API-key auth and a hardened wide-bind mode show that the
absence of auth on network exposure is still a known, unsettled request rather than
an enforced transition
([auth request #8536](https://github.com/ollama/ollama/issues/8536),
[secure-mode request #11941](https://github.com/ollama/ollama/issues/11941)).

### Limits of the survey

- “No public incident found” means none was visible in upstream advisories,
  changelogs, issues, and commit history searched for this ticket; it does not prove
  none was privately reported.
- GoTTY's age makes it evidence of an older design, not evidence of current browser
  or maintenance practice.
- The projects protect different assets: Vite primarily exposes source, Ollama model
  operations/inference, and Jupyter/editor/terminal tools expose direct code
  execution. Their controls are recorded here as facts, not ranked as interchangeable
  answers.
