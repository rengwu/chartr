package server_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/rengwu/chartr/internal/chartrtest"
	"github.com/rengwu/chartr/internal/config"
	"github.com/rengwu/chartr/internal/model"
)

func TestSnapshotCarriesResolvedNotifyPrefs(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", `
after = "2m"
settle = "3s"
enabled = false
`)

	register(t, h, repo)
	got := h.Snapshot(ctx(t)).Notify
	if got.After != "2m0s" || got.Settle != "3s" || got.Enabled {
		t.Errorf("snapshot notify prefs = %+v, want 2m0s / 3s / disabled", got)
	}
}

func TestSnapshotBadNotifyValueWarnsAndKeepsDefault(t *testing.T) {
	h := chartrtest.Start(t)
	repo := chartrtest.NewSpaceRepo(t)
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", `after = "tomorrow"`)

	resp := register(t, h, repo)
	snap := h.Snapshot(ctx(t))
	if snap.Notify.After != config.DefaultNotifyAfter.String() {
		t.Errorf("bad after resolved to %q, want default %q", snap.Notify.After, config.DefaultNotifyAfter)
	}
	warnings := findSpace(t, snap, resp.ID).Warnings
	if len(warnings) != 1 || !strings.Contains(warnings[0], "notify.toml") ||
		!strings.Contains(warnings[0], "after") {
		t.Errorf("bad notify warning = %v, want one naming file and key", warnings)
	}
}

func TestNotifyPrefsRereadByNotice(t *testing.T) {
	h := chartrtest.Start(t)
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", `after = "30s"`)
	register(t, h, chartrtest.NewSpaceRepo(t))
	if got := h.Snapshot(ctx(t)).Notify.After; got != "30s" {
		t.Fatalf("notify after starts at %q, want 30s", got)
	}

	cc := h.DialControl(ctx(t))
	defer cc.Close()
	cc.ReadSnapshot(ctx(t))
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", `after = "2m"`)
	cc.WaitFor(ctx(t), func(m model.Model) bool { return m.Notify.After == "2m0s" })
}

func TestNotifyConfigIsAnOpenableGlobalLayer(t *testing.T) {
	h := chartrtest.Start(t)
	chartrtest.WriteFile(t, h.ConfigDir, "notify.toml", `after = "90s"`)
	register(t, h, chartrtest.NewSpaceRepo(t))

	l := layer(t, h.Snapshot(ctx(t)).Config, "notify-config")
	if want := filepath.Join(h.ConfigDir, "notify.toml"); l.Path != want {
		t.Errorf("notify config path = %q, want %q", l.Path, want)
	}
	if l.Holds != "notifications" || !l.Exists {
		t.Errorf("notify config layer = %+v, want it holding notifications and existing", l)
	}

	record := stubEditor(t)
	code, body := h.Post("/api/config/open", map[string]string{"layer": "notify-config"})
	if code != 200 {
		t.Fatalf("open notify-config = %d, body %s", code, body)
	}
	if got := waitForFile(t, record); !strings.Contains(got, l.Path) {
		t.Errorf("editor received %q, want %q", got, l.Path)
	}
}

func TestCreateStampsNotifyConfigFromDefaults(t *testing.T) {
	h := chartrtest.Start(t)
	register(t, h, chartrtest.NewSpaceRepo(t))

	path := filepath.Join(h.ConfigDir, "notify.toml")
	if layer(t, h.Snapshot(ctx(t)).Config, "notify-config").Exists {
		t.Fatal("notify-config reported existing before create")
	}
	code, body := h.Post("/api/config/create", map[string]string{"layer": "notify-config"})
	if code != 200 || !strings.Contains(body, `"created":true`) {
		t.Fatalf("create notify-config = %d, body %s", code, body)
	}
	got, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read created notify.toml: %v", err)
	}
	if string(got) != string(config.ScaffoldNotifyTOML) {
		t.Error("created notify.toml is not the bundled scaffold verbatim")
	}
	if !layer(t, h.Snapshot(ctx(t)).Config, "notify-config").Exists {
		t.Error("notify-config still reported missing after create")
	}
}
