# Native deb and rpm packages use the host's WebKitGTK

Linux releases carry two more supported desktop artifacts per architecture: a
deb for Ubuntu 24.04+/Debian 13+, and an rpm tested on current Fedora. They
contain the same cgo webview shell and embedded frontend as the AppImage, plus a
desktop entry, AppStream metadata, icon and license. Unlike the AppImage, they
contain no GTK or WebKit libraries. Package metadata depends on WebKitGTK 4.1,
letting apt or dnf install the renderer and deliver its security updates.

This amends [0011](0011-one-supported-artifact-tiered-extras.md) a second time:
the native Linux packages join the AppImage on the supported tier. A release is
gated on building amd64 and arm64 packages, installing each one in its target
distribution family, and screenshot-testing that its window rendered the
cockpit. The rpm is deliberately tested on Fedora even though the executable is
built on the Ubuntu runner; this turns their shared WebKit soname and compatible
glibc baseline into an assertion rather than an assumption.

nFPM assembles both formats from one manifest. It does not build the executable
and does not choose dependencies: `make linux-packages` first invokes the
existing native `make webview` build, and the checked-in manifest names the
Debian- and RPM-family WebKit packages explicitly. nFPM itself is version-pinned
so rebuilding a tag cannot silently change archive metadata.

## Consequences

- The native downloads are small and integrate with the application menu and
  system package database; WebKit fixes arrive with normal system updates.
- They are not universal. The deb intentionally starts at Ubuntu 24.04 or
  Debian 13, whose system libraries meet its WebKitGTK 4.1 and glibc 2.39
  baseline, while the AppImage remains the answer for older and otherwise
  unsupported distributions.
- GitHub Releases remains the distribution service. These files are directly
  installable packages, not an apt or dnf repository, so installing a later
  chartr version is not automatic yet.
- Packaging failures now fail a tag. Windows and macOS loose shells remain the
  best-effort tier described by 0011 and 0016.
