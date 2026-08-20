//go:build webview && darwin

package main

/*
#cgo CFLAGS: -x objective-c -Wno-deprecated-declarations
#cgo LDFLAGS: -framework Cocoa

#import <Cocoa/Cocoa.h>

// wfSetAppIcon points the Dock tile at the chartr mark.
//
// This is the LOOSE shell's only surface for it. A bare binary has no bundle, so
// there is no CFBundleIconFile for AppKit to read and no icon in Finder at all;
// setApplicationIconImage: dresses the Dock tile for the life of the process and
// nothing beyond it.
//
// `make bundle` now assembles a real chartr.app whose property list points at an
// .icns downscaled from this very same PNG (ADR 0016), so inside the bundle
// Finder, the Dock and the app switcher are already dressed before this runs and
// the call is a no-op over identical artwork. The loose shell is still a shipped
// artifact, so this stays.
//
// AppKit does not mask what it is handed here any more than Finder masks an .icns,
// which is why the PNG behind this is the mac-specific master carrying Apple's own
// squircle and inset rather than the square PWA icon.
//
// It must run after the NSApplication exists, which is why the Go side calls it
// beside installNativeMenu rather than beside setAppName.
//
// dataWithBytes: copies, so the Go slice behind `bytes` is not retained past the
// call and the pointer stays cgo-legal.
static void wfSetAppIcon(const void *bytes, int len) {
  @autoreleasepool {
    NSData *data = [NSData dataWithBytes:bytes length:(NSUInteger)len];
    NSImage *img = [[NSImage alloc] initWithData:data];
    if (img != nil) {
      [[NSApplication sharedApplication] setApplicationIconImage:img];
    }
  }
}
*/
import "C"

import (
	"unsafe"

	webview "github.com/webview/webview_go"
)

// appIconPath is the mac-specific master rather than the square icon-512.png
// the PWA manifest points at, because setApplicationIconImage: does not mask
// either: what the PNG says, the Dock draws. Apple's shape has to be in the
// pixels, and it is only in this one. It is the same bytes `make bundle`
// downscales into the bundle's .icns (ADR 0016), so the loose shell's Dock tile
// and the bundle's Finder icon can never drift apart.
const appIconPath = "icon-mac-1024.png"

// setAppIcon dresses the Dock tile through NSApplication, which is process-wide
// rather than per-window, so the webview handle goes unused here — Linux is the
// platform that needs it (icon_linux.go).
func setAppIcon(_ webview.WebView, png []byte) {
	if len(png) == 0 {
		return
	}
	C.wfSetAppIcon(unsafe.Pointer(&png[0]), C.int(len(png)))
}
