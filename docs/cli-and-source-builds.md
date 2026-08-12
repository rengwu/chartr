# CLI and source builds

## Run the CLI

Download an archive from the
[releases page](https://github.com/rengwu/chartr/releases), unpack it and put
`chartr` on your `PATH`.

```sh
chartr                 # Opens at http://127.0.0.1:8787
chartr -addr :9000     # Listen on another address
chartr -data-dir ~/w   # Set the session root
```

chartr has no authentication. Keep it bound to `127.0.0.1` unless you intend to
make it accessible over a network. See [Security](../SECURITY.md) for details.

Agent CLIs are installed separately; chartr does not bundle them.

## Build from source

Requires Go 1.26+ and Node 22+.

```sh
make build
make check
make test
make dmg
```

On Linux, `make appimage` builds the self-contained desktop artifact and
`make linux-packages` builds native deb and rpm packages. Both require the GTK3
and WebKitGTK 4.1 development packages; native packages use the installed
WebKitGTK runtime rather than bundling it.
