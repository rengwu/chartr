#!/usr/bin/env python3
"""Redirect a bundled libwebkit2gtk's hardcoded helper-process directory.

WebKitGTK is not one library. It spawns WebKitNetworkProcess and
WebKitWebProcess as separate executables and loads an injected bundle, and it
finds all of them through a directory baked into the library at compile time --
`/usr/lib/<triplet>/webkit2gtk-4.1`. Inside an AppImage that path is either
absent or, worse, holds the host's WebKit rather than the one we shipped, and
the failure is not a link error at startup: the window opens and renders
"WebKit encountered an internal error" because a helper could not be spawned.

Until WebKitGTK 2.46 the environment variable WEBKIT_EXEC_PATH overrode this.
It is gone from 2.52 -- the library exports no knob for it at all (the only
survivor is WEBKIT_INJECTED_BUNDLE_PATH) -- so the string itself is what has to
change, and this script is that edit.

The replacement is written over the original bytes and NUL-terminated, never
inserted, so every offset in the file stays exactly where it was: the ELF is
not relocated, no section shifts, and nothing else needs patching. That is the
whole reason the new path must be *shorter* than the old one, which is enforced
below rather than assumed.
"""

import argparse
import sys


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("library", help="the bundled libwebkit2gtk-4.1.so.0 to edit in place")
    ap.add_argument("old", help="the compiled-in helper directory, e.g. /usr/lib/x86_64-linux-gnu/webkit2gtk-4.1")
    ap.add_argument("new", help="the directory AppRun will make resolve to the AppDir's helpers")
    args = ap.parse_args()

    old = args.old.encode()
    new = args.new.encode()

    if len(new) > len(old):
        print(
            f"relocate-webkit: {args.new!r} ({len(new)} bytes) is longer than "
            f"{args.old!r} ({len(old)} bytes); the replacement is written in place "
            f"and cannot grow the file",
            file=sys.stderr,
        )
        return 1

    with open(args.library, "rb") as fh:
        blob = fh.read()

    hits = blob.count(old)
    if hits == 0:
        # Failing here is the point. A WebKitGTK that changed this string would
        # otherwise produce an AppImage that builds, ships, launches, and shows
        # an error page instead of the cockpit -- caught by nobody until an
        # operator downloaded it.
        print(
            f"relocate-webkit: {args.old!r} does not appear in {args.library} -- "
            f"this WebKitGTK does not lay its helpers out where we expect, and the "
            f"AppImage would ship with a broken renderer",
            file=sys.stderr,
        )
        return 1

    # Both the helper directory and the injected-bundle path share this prefix,
    # so one substitution redirects both onto the single symlink AppRun makes.
    patched = blob.replace(old, new + b"\0" * (len(old) - len(new)))

    with open(args.library, "wb") as fh:
        fh.write(patched)

    print(f"relocate-webkit: redirected {hits} path(s) from {args.old} to {args.new}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
