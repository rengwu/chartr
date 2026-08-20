//go:build webview && linux

package main

/*
#cgo pkg-config: gtk+-3.0

#include <gtk/gtk.h>
#ifdef GDK_WINDOWING_WAYLAND
#include <gdk/gdkwayland.h>
#endif

// wfSetAppIcon sets the given GtkWindow's icon from PNG bytes.
//
// An AppImage is not installed anywhere a window manager can resolve a
// .desktop file's Icon= from WM_CLASS — that lookup is what a deb/rpm install
// gets for free from the hicolor entries nfpm.yaml stages — so without this,
// launching the AppImage (or running the loose binary straight off disk) drew
// a window with no icon at all in the taskbar or alt-tab switcher.
//
// gtk_window_set_default_icon looked like the natural fit (it is the closest
// thing GTK has to macOS's process-wide NSApplication call), but it only seeds
// windows created *after* it runs, and webview_go's window already exists by
// the time applyAppIcon does — the icon never appeared. gtk_window_set_icon on
// the actual instance, from the GtkWindow pointer webview_go's Window() hands
// back, has no such ordering requirement.
//
// GdkPixbufLoader turns the PNG bytes into the GdkPixbuf the icon setter wants;
// gdk-pixbuf is already part of gtk+-3.0's own dependency chain, so this needs
// no pkg-config line beyond the one above.
static void wfSetAppIcon(void *window, const void *bytes, int len) {
  GtkWindow *gtk_window = GTK_WINDOW(window);

  // X11 shells associate the window with chartr.desktop through WM_CLASS.
  // The explicit value also avoids inheriting an AppImage filename as the
  // class on runtimes which derive it from argv[0].
  gtk_window_set_wmclass(gtk_window, "chartr", "chartr");

  // A GdkWindow exists after realization. On Wayland its application ID is
  // the only supported way for the compositor to associate this surface with
  // chartr.desktop and therefore with the icon bundled in the AppImage.
  gtk_widget_realize(GTK_WIDGET(gtk_window));
#ifdef GDK_WINDOWING_WAYLAND
  GdkWindow *gdk_window = gtk_widget_get_window(GTK_WIDGET(gtk_window));
  if (gdk_window != NULL && GDK_IS_WAYLAND_WINDOW(gdk_window)) {
    gdk_wayland_window_set_application_id(gdk_window, "chartr");
  }
#endif

  GdkPixbufLoader *loader = gdk_pixbuf_loader_new();
  GError *err = NULL;
  if (gdk_pixbuf_loader_write(loader, (const guchar *)bytes, (gsize)len, &err)) {
    gdk_pixbuf_loader_close(loader, NULL);
    GdkPixbuf *pixbuf = gdk_pixbuf_loader_get_pixbuf(loader);
    if (pixbuf != NULL) {
      gtk_window_set_icon(gtk_window, pixbuf);
    }
  } else {
    gdk_pixbuf_loader_close(loader, NULL);
  }
  if (err != NULL) {
    g_error_free(err);
  }
  g_object_unref(loader);
}
*/
import "C"

import (
	"unsafe"

	webview "github.com/webview/webview_go"
)

// appIconPath is the square, full-bleed PWA master — the same bytes
// packaging/linux/nfpm.yaml installs into the hicolor theme and the Makefile's
// `appimage` target (APPIMAGE_MARK) hands linuxdeploy for the AppImage's own
// launcher icon — rather than the mac-specific squircle set in icon_darwin.go
// (ADR 0016). Linux desktops mask, round and theme icons themselves, and feeding
// them Apple's pre-inset, pre-shadowed art would draw a small tile floating in a
// transparent box.
const appIconPath = "icon-512.png"

func setAppIcon(w webview.WebView, png []byte) {
	win := w.Window()
	if win == nil || len(png) == 0 {
		return
	}
	C.wfSetAppIcon(win, unsafe.Pointer(&png[0]), C.int(len(png)))
}
