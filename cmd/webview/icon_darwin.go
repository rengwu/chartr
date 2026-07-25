//go:build webview && darwin

package main

/*
#cgo CFLAGS: -x objective-c -Wno-deprecated-declarations
#cgo LDFLAGS: -framework Cocoa

#import <Cocoa/Cocoa.h>

// wfSetAppIcon points the Dock tile at the chartr mark.
//
// A bare binary has no bundle, so there is no CFBundleIconFile for AppKit to
// read and no icon in Finder — setApplicationIconImage: is the only surface a
// non-bundled app can dress, and it dresses the Dock tile for the life of the
// process. Handing the shell a real .app is a packaging change (ADR 0011 keeps
// this tier best-effort and unbundled), not something this can reach.
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
