# Prebuilt embedded native plugins

Native plugins are trusted libraries, loaded only when a pane opens. They use
`kind = "embedded"`; `kind = "native"` still means a GPUI module linked at Chartr
build time and cannot be installed separately.

Installation **never compiles anything** or executes package hooks. Authors
build release artifacts in their own CI. Users install from Settings → Plugins
using a repository URL or a prepared local package directory.

```toml
manifest_version = 2
id = "com.example.viewer"
name = "Viewer"
version = "1.0.0"
kind = "embedded"
icon = "ViewIcon"
release = "https://github.com/example/viewer/releases/latest/download"

[libraries]
linux-x86_64 = "libviewer.so"
macos-aarch64 = "libviewer.dylib"

[capabilities]
multiplicity = "multiple"
restorable = true
```

The optional `release` is an HTTPS directory. When the package does not already
contain its platform library, the installer downloads
`chartr-plugin-<os>-<architecture>.tar.gz` and the adjacent `.sha256` file.
Architecture names are Rust's `std::env::consts::ARCH` (`x86_64`, `aarch64`).
The checksum file contains `HEX_DIGEST  ARCHIVE_FILENAME`. The archive root
contains the complete manifest, icons, library and its private runtime files.
It may contain only directories and regular files. Paths cannot escape staging;
privileged permission bits are removed. An absent platform asset is an error.
Checksums detect corruption; the user still needs to trust the publisher.

The generic SDK is [chartr-native-plugin](../crates/chartr-native-plugin/src/lib.rs),
which has no GPUI dependency. It defines ABI v1 and its JSON message schema.
A plugin exports `chartr_native_plugin_v1`, returning a static function table.
Use `extern "C"`, C-compatible layout, explicit pointer/length byte spans and
opaque handles. Catch panics; never unwind through the host.

All host calls run on the UI thread. Host callbacks may run on any thread but
must stop before `destroy` returns. Plugin code must finish using the borrowed
parent before then. `resize` receives logical top-left coordinates and physical
scale; Linux uses an X11/XWayland parent XID, macOS an NSView pointer. Initialize
engines lazily and own any helper processes inside the package. `shutdown` runs
once after all views close, on the initialization thread.

`State` publishes a title, up to 32 toolbar controls and up to 64 shortcuts.
Controls are buttons (`value = null`) or text inputs; icons are package-relative
SVGs. `Action` returns the control ID and submitted text. `Focus` selects a host
control or reports that native content gained focus. `Wake` asks the host to
call `Poll`, letting plugins drain their own event queue without an idle timer.
The host sends theme changes so local plugin content can match the interface.

On Linux, system packages may place the complete plugin under
`/usr/lib/chartr/plugins/<id>`. On macOS the system root is
`/Library/Application Support/chartr/plugins`. User-installed packages take
precedence. Native code runs with the user's authority; web permission grants
do not constrain it. Disabling a plugin closes its views; libraries remain
mapped until application exit.
