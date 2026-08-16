package env

import (
	"os"
	"strings"
)

// snapshotMarker is exported by the AppImage's AppRun (packaging/linux/
// AppRun) when it has saved the host's values of the variables it is about to
// point at the bundle. The marker is what makes the snapshot variables
// decidable: without it, "no CHARTR_HOST_LD_LIBRARY_PATH" cannot tell "AppRun
// overwrote a variable the operator never set" apart from "not running from
// the bundle at all", and only the first may be restored to unset.
const snapshotMarker = "CHARTR_HOST_SNAPSHOT"

// snapshotPrefix prefixes AppRun's saved copy of each variable in bundleVars.
const snapshotPrefix = "CHARTR_HOST_"

// bundleVars is every variable AppRun aims at the AppDir, in the order AppRun
// lists them. The two lists are one fact stated twice — keep them in sync.
var bundleVars = []string{
	"LD_LIBRARY_PATH",
	"GIO_MODULE_DIR",
	"GDK_PIXBUF_MODULE_FILE",
	"GSETTINGS_SCHEMA_DIR",
	"XDG_DATA_DIRS",
	"WEBKIT_INJECTED_BUNDLE_PATH",
	"WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS",
}

// HostEnviron returns the environment to hand a process chartr launches on
// the operator's behalf: the folder chooser, a terminal shell, an agent, a
// browser, a notification.
//
// The AppImage's AppRun repoints the variables above at the bundled
// libraries so WebKitGTK can run, and without this every process chartr
// spawns would inherit that. A host binary started under it loads the
// bundle's libraries against its own and dies in the loader before main —
// the AppImage's zenity, curl and gio all failing with symbol errors is the
// reported shape of it. AppRun snapshots the operator's own values first;
// this restores them: a snapshotted variable goes back to exactly what the
// operator had, empty included, and one they never set goes back to unset.
//
// chartr's own environment is never mutated — WebKit's helper processes are
// spawned by the library out of this process and must keep the bundle paths.
// Outside the bundle there is no snapshot and the environment passes through
// untouched, so the plain binary and the native packages are unaffected.
func HostEnviron() []string {
	return restoreHost(os.Environ(), os.LookupEnv)
}

// restoreHost is HostEnviron's pure core, taking the environment and the
// snapshot lookup as parameters so a test can drive both.
func restoreHost(env []string, lookup func(string) (string, bool)) []string {
	if _, ok := lookup(snapshotMarker); !ok {
		return env
	}
	out := make([]string, 0, len(env))
	for _, kv := range env {
		name, _, _ := strings.Cut(kv, "=")
		// The marker and the snapshots themselves are chartr's bookkeeping,
		// not the operator's environment; a child re-running the bundle
		// snapshots afresh from the restored values.
		if name == snapshotMarker || strings.HasPrefix(name, snapshotPrefix) {
			continue
		}
		if isBundleVar(name) {
			continue
		}
		out = append(out, kv)
	}
	for _, v := range bundleVars {
		if val, ok := lookup(snapshotPrefix + v); ok {
			out = append(out, v+"="+val)
		}
	}
	return out
}

func isBundleVar(name string) bool {
	for _, v := range bundleVars {
		if name == v {
			return true
		}
	}
	return false
}
