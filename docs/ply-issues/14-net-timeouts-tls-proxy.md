# Feature request: `net` HTTP timeouts, TLS roots, proxy, and cancel/eviction semantics

**Type:** Feature request + docs gap
**Severity:** High — a hung or failed request can disable a feed for the whole session
**Status:** Ready (validated against Ply 1.1.1 and `ureq` 3.3.0 source; Windows behaviour not run)

## Summary

`ply_engine::net` HTTP requests on native targets cannot be configured. Each
request builds `ureq::Agent::new_with_defaults()` on its own thread, and
`HttpConfig` offers only headers and a body. That has four consequences:

1. **No timeouts.** A stalled connection stays `Pending` forever.
2. **No choice of TLS roots.** Only the bundled Mozilla roots are trusted, so
   networks that re-sign TLS with a CA in the OS store fail.
3. **No system proxy** beyond `*_PROXY` environment variables.
4. **Retry is hard to get right.** Because of the same-id no-op and the
   eviction rules, a naive "fire again on error" loop never re-fires.

Issues #6 and #7 ask for the same-id and 60-frame behaviour to be documented.
This issue adds the configuration gap and records what `cancel()` does, which
changes how #6 and #7 should be written.

## What we observed

Source references are `ply-engine-1.1.1/src/net/` unless stated.

### 1. No timeouts

```rust
// mod.rs:107-109
std::thread::spawn(move || {
    let result: Result<Response, String> = (|| {
        let agent = ureq::Agent::new_with_defaults();
```

`ureq-3.3.0/src/config.rs` (`impl Default for Timeouts`) sets every timeout to
`None` except `await_100` (1 s). `HttpConfig` (`http.rs:7`) has `header`,
`body` and `body_bytes` only.

Pending requests are never evicted:

```rust
// mod.rs:59
HttpRequestState::Pending(_) => true, // never evict
```

So after a sleep/resume, a Wi-Fi roam or a VPN change, a request on a dead
TCP connection can block its thread indefinitely (Windows keepalive defaults
to about two hours), and the app has no way to stop waiting except
`Request::cancel()`, which does not stop the thread (see below). One thread is
spawned per request with no cap (`mod.rs:107`).

### 2. TLS roots

`Cargo.toml:164-167` enables `ureq` with `features = ["rustls"]`,
`default-features = false`. In `ureq` 3.3.0 that feature means
`rustls-no-provider` + `_ring` + `rustls-webpki-roots`, and
`TlsConfig::default()` (`tls/mod.rs`) sets `root_certs: RootCerts::WebPki`.

`RootCerts::PlatformVerifier` exists but needs ureq's `platform-verifier`
feature **and** an explicit config. Enabling the feature from a downstream
crate does not change the default, so downstream apps cannot opt in without a
fork or patch.

Impact: corporate or antivirus TLS inspection (a CA installed in the Windows
store) makes every HTTPS request fail with an unknown-issuer error. The app
sees only the error string.

### 3. Proxy

`Config::default()` takes the proxy from `Proxy::try_from_env()`. On Windows,
`ureq` can also read the registry (`ProxyEnable`/`ProxyServer`) but only with
its `win-system-proxy` feature (`proxy.rs:241-265`). A downstream crate *can*
enable that feature itself and it will take effect, but it reads only
`ProxyServer` as `http://{value}`: no per-protocol form, no PAC. A GUI app
launched from Explorer rarely has `HTTPS_PROXY` set.

### 4. Same-id no-op, eviction, and `cancel()`

```rust
// mod.rs:91
// Idempotent: don't re-fire if a request with this ID already exists
```

`clean()` (`mod.rs:41-65`) keeps `Pending` forever and evicts `Done`/`Error`
once `frames_not_accessed > max_frames_not_used` (60, `mod.rs:36`). Both
`net::request(id)` and `Request::response()` reset `frames_not_accessed`
to 0.

Consequence for a polling app: the usual loop calls `net::request(id)` every
frame. A stored `Err` is touched every frame, so it is never evicted, and
`net::get(id, ..)` stays a no-op. The feed never retries. Parse-on-`Done`
code also re-runs every frame on the same cached body.

There **is** a way out, which is not documented in the module docs:

```rust
// http.rs:115
/// Cancel and remove a pending request. Consumes the handle.
pub fn cancel(self) {
    let mut mgr = super::NET_MANAGER.lock().unwrap();
    mgr.http_requests.remove(&self.id);
}
```

Despite the "pending" wording, it removes the entry in any state, so the id
becomes fireable again. It does not stop the worker thread. Its `Sender`
result goes to a dropped `Receiver` and is discarded when the thread finally
returns, so a request that hangs forever leaks its thread.

## What needs to change

1. **Per-request configuration on `HttpConfig`**, at least:
   ```rust
   net::get("alerts", url, |c| c
       .timeout(Duration::from_secs(30))          // whole request
       .connect_timeout(Duration::from_secs(10))
       .header("User-Agent", "myapp/1.0"));
   ```
   Map to `ureq`'s `timeout_global`, `timeout_connect` and `timeout_recv_body`.
   Alternatively a sensible non-`None` default (for example 10 s connect,
   30 s global), documented and overridable.
2. **TLS root selection**, either an `HttpConfig`/global option
   (`webpki` vs `platform`) or a default of the platform verifier on desktop
   with webpki as fallback. Document the trade-off: Mozilla roots are
   independent of a possibly intercepted OS store; the OS store trusts
   enterprise CAs.
3. **System proxy.** Enable `ureq`'s `win-system-proxy` in Ply's own
   dependency line (a one-word change), and document that PAC is unsupported.
4. **Document `cancel()` and the retry pattern** in the `net` module docs, and
   fix the doc comment ("pending" is wrong). A short example:
   ```rust
   match net::request("alerts") {
       None => { /* fire (subject to backoff) */ }
       Some(req) => match req.response() {
           None => { /* pending */ }
           Some(Ok(resp)) => { handle(resp); req.cancel(); }   // consume
           Some(Err(e))   => { log(e); req.cancel(); /* back off */ }
       },
   }
   ```
5. **Optional:** a free function `net::cancel(id)`, so a caller can drop an
   entry without first getting a handle. Longer term, an abortable worker
   (for example a tokio task with `AbortHandle`, or a thread that observes a
   cancel flag) so `cancel()` actually frees the socket, plus a thread cap.
6. **Optional:** make `max_frames_not_used` configurable (see #7) and
   `debug`-log when an unconsumed response is evicted.

## Why

Any app that polls a live feed hits this on the first dropped connection. We
hit it as: alerts, borders and hurricane data stop updating and the status
stays stale with no indication, because the fetch never completes and the
"already fired" guard never clears. On a TLS-inspecting network every request
fails and, combined with the same-id no-op, stays failed until restart.

## Workarounds (what we do today)

- App-side deadline per request, then `cancel()` and exponential backoff.
- `ureq = { version = "3", default-features = false, features = ["win-system-proxy"] }`
  in our own `Cargo.toml` to turn on the registry proxy via feature
  unification.
- Planned: move our fetches off `net` onto `reqwest`, which is already in our
  build through another dependency and uses the platform verifier and system
  proxy. If Ply gains the options above we would not need to.

## Notes

- Every statement about `ureq` is from 3.3.0 (the version in our lock).
  3.4.0 to 3.4.2 add TLS-through-CONNECT-proxy support and fix timeout
  budgets and the connect-phase TLS handshake; they do not change the
  defaults above. Ply's `ureq = "3"` already permits them.
- Related: #6 (same-id behaviour) and #7 (60-frame eviction). Suggest filing
  these together or cross-linking, and correcting #6/#7 where they say there
  is no remove API.
- Not tested on Windows. The Windows registry proxy behaviour and the
  platform-verifier failure mode are inferred from source.
