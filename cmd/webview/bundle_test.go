package main

import (
	"path/filepath"
	"testing"
)

// A bundled launch is the launch nobody watches: no terminal, and a stderr
// stream Finder discards. What decides whether it lives — where it writes, and
// whether a window-server argument is fatal — is pure path work, so it is tested
// here at CGO_ENABLED=0 with constructed paths and no bundle in existence.

func TestIsBundled(t *testing.T) {
	tests := []struct {
		name string
		exe  string
		want bool
	}{
		{"installed bundle", "/Applications/chartr.app/Contents/MacOS/chartr", true},
		{"bundle anywhere else", filepath.Join("/Users/op/build/mac", "chartr.app", "Contents", "MacOS", "chartr"), true},
		{"loose shell in a bin dir", "/usr/local/bin/chartr-shell", false},
		{"loose shell in the build output", "/Users/op/src/chartr/build/shell/chartr-shell_v1_darwin_arm64", false},
		{"bare name, launched off PATH", "chartr-shell", false},
		{"no path at all", "", false},
		// The shape is the whole predicate, so the near-misses matter: a `MacOS`
		// directory with no `Contents` above it is not a bundle, and neither is
		// an executable sitting anywhere else inside one.
		{"MacOS dir outside a bundle", "/Users/op/MacOS/chartr", false},
		{"Contents/MacOS under a plain directory", "/Users/op/chartr/Contents/MacOS/chartr", false},
		{"executable sitting in Contents", "/Applications/chartr.app/Contents/chartr", false},
		{"executable sitting beside the bundle", "/Applications/chartr.app/chartr", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := isBundled(tt.exe); got != tt.want {
				t.Errorf("isBundled(%q) = %v, want %v", tt.exe, got, tt.want)
			}
		})
	}
}

func TestRuntimeRoot(t *testing.T) {
	const (
		bundledExe = "/Applications/chartr.app/Contents/MacOS/chartr"
		looseExe   = "/usr/local/bin/chartr-shell"
		configRoot = "/Users/op/.config/chartr"
	)

	// appImageExe is what the AppImage runtime actually hands the shell: a mount
	// point with a suffix chosen afresh every launch, which is exactly why the
	// path cannot be the signal and the runtime's APPIMAGE variable is.
	const appImageExe = "/tmp/.mount_chartrAbC123/usr/bin/chartr"
	inAppImage := map[string]string{"APPIMAGE": "/home/op/Downloads/chartr.AppImage"}

	tests := []struct {
		name       string
		explicit   string
		exe        string
		configRoot string
		env        map[string]string
		want       string
	}{
		// The terminal launch is unchanged: empty means the working directory,
		// which is what the operator cd'd to.
		{"loose launch keeps the working directory", "", looseExe, configRoot, nil, ""},
		// The whole reason this ticket exists: Finder hands a bundle `/`, and the
		// lock would die on it before a window existed.
		{"bundled launch anchors to the config root", "", bundledExe, configRoot, nil, configRoot},
		// One root, however the operator started the app.
		{"explicit root wins when bundled", "/Users/op/work", bundledExe, configRoot, nil, "/Users/op/work"},
		{"explicit root wins when loose", "/Users/op/work", looseExe, configRoot, nil, "/Users/op/work"},
		// No home to anchor to: the config root itself degrades to the runtime
		// root here, and so does this.
		{"bundled with no home degrades to the working directory", "", bundledExe, "", nil, ""},

		// The AppImage half (#3). The failure it replaces is quiet rather than
		// fatal — a lock and a session archive per directory the operator
		// happened to launch from — so the anchor matters just as much.
		{"appimage launch anchors to the config root", "", appImageExe, configRoot, inAppImage, configRoot},
		{"explicit root wins in an appimage", "/home/op/work", appImageExe, configRoot, inAppImage, "/home/op/work"},
		{"appimage with no home degrades to the working directory", "", appImageExe, "", inAppImage, ""},
		// The mount-point path on its own proves nothing: without the runtime
		// saying so this is just a binary someone ran out of /tmp, and it keeps
		// the working directory it was started in.
		{"a mount-point path alone is not an appimage", "", appImageExe, configRoot, nil, ""},
		// An empty APPIMAGE is not a statement that we are in one.
		{"an empty APPIMAGE is not an appimage", "", appImageExe, configRoot, map[string]string{"APPIMAGE": ""}, ""},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			lookup := func(name string) (string, bool) {
				v, ok := tt.env[name]
				return v, ok
			}
			if got := runtimeRoot(tt.explicit, tt.exe, tt.configRoot, lookup); got != tt.want {
				t.Errorf("runtimeRoot(%q, %q, %q, %v) = %q, want %q",
					tt.explicit, tt.exe, tt.configRoot, tt.env, got, tt.want)
			}
		})
	}
}

func TestParseFlags(t *testing.T) {
	t.Run("a terminal launch still refuses an unrecognised argument", func(t *testing.T) {
		if _, err := parseFlags([]string{"-psn_0_123456"}, false); err == nil {
			t.Fatal("parseFlags accepted an unknown flag from a terminal launch; want an error")
		}
	})

	t.Run("a bundled launch ignores what the window server injects", func(t *testing.T) {
		got, err := parseFlags([]string{"-psn_0_123456"}, true)
		if err != nil {
			t.Fatalf("parseFlags: %v, want a bundled launch to tolerate it", err)
		}
		if got != (shellFlags{}) {
			t.Errorf("flags = %+v, want the defaults", got)
		}
	})

	t.Run("a bundled launch keeps the flags it did recognise", func(t *testing.T) {
		got, err := parseFlags([]string{"-data-dir", "/Users/op/work", "-psn_0_123456"}, true)
		if err != nil {
			t.Fatalf("parseFlags: %v", err)
		}
		if got.dataDir != "/Users/op/work" {
			t.Errorf("dataDir = %q, want the explicitly passed root", got.dataDir)
		}
	})

	t.Run("ordinary parsing is unchanged", func(t *testing.T) {
		got, err := parseFlags([]string{"-data-dir", "/Users/op/work", "-version"}, false)
		if err != nil {
			t.Fatalf("parseFlags: %v", err)
		}
		if got.dataDir != "/Users/op/work" || !got.showVersion {
			t.Errorf("flags = %+v, want both flags read", got)
		}
	})
}
