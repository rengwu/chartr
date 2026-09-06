# Plugin examples

These reference examples are not bundled with Chartr and do not appear in its
default launcher or Settings catalog.

- `clock` is a portable web package. To try it, install this directory through
  Settings → Plugins → Local folder.
- `hello` demonstrates a build-time native module. It can be checked independently
  with `cargo check --manifest-path examples/plugins/hello/Cargo.toml --locked`
  from the repository root. Chartr does not support installing native modules
  as external packages; using it in the app requires explicit build-time wiring.
