//go:build webview && linux

package main

/*
#cgo pkg-config: glib-2.0

#include <glib.h>
#include <stdlib.h>

static void wfSetAppName(const char *name) {
  // GTK uses the program name as the Wayland application ID and as the
  // default X11 resource name. Set both before webview_go calls gtk_init so
  // they match chartr.desktop regardless of the AppImage's filename.
  g_set_prgname(name);
  g_set_application_name(name);
}
*/
import "C"

import "unsafe"

const missingRuntime = "no WebKitGTK or no display — check libwebkit2gtk and $DISPLAY/$WAYLAND_DISPLAY"

func setAppName(name string) {
	cname := C.CString(name)
	defer C.free(unsafe.Pointer(cname))
	C.wfSetAppName(cname)
}

func installNativeMenu(string) {}

func raiseInstance(int) bool { return false }
