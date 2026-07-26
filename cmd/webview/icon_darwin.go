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

import "unsafe"

func setAppIcon(png []byte) {
	if len(png) == 0 {
		return
	}
	C.wfSetAppIcon(unsafe.Pointer(&png[0]), C.int(len(png)))
}
