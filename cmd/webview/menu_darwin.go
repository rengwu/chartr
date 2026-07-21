//go:build webview && darwin

package main

/*
#cgo CFLAGS: -x objective-c -Wno-deprecated-declarations
#cgo LDFLAGS: -framework Cocoa

#import <Cocoa/Cocoa.h>

// wfItem appends one menu item. Every action here is a standard responder-chain
// selector with a target of nil, which is why the menu needs no callback into
// Go: NSApplication handles the app items, and WKWebView — the first responder
// inside the window — handles reload: and the edit items itself.
static NSMenuItem *wfItem(NSMenu *menu, NSString *title, SEL action, NSString *key, NSUInteger mask) {
  NSMenuItem *item = [menu addItemWithTitle:title action:action keyEquivalent:key];
  [item setKeyEquivalentModifierMask:mask];
  return item;
}

static NSMenu *wfSubmenu(NSMenu *bar, NSString *title) {
  NSMenuItem *item = [bar addItemWithTitle:title action:NULL keyEquivalent:@""];
  NSMenu *menu = [[NSMenu alloc] initWithTitle:title];
  [item setSubmenu:menu];
  return menu;
}

// wfInstallMenu gives the bare webview window back the OS affordances a browser
// tab had for free: Quit, Reload, and the edit items. Deliberately minimal —
// ADR 0013 declines a dock badge and a URL scheme, and this menu is the whole
// of the shell's native integration beyond the window itself.
static void wfInstallMenu(const char *cname) {
  NSString *name = [NSString stringWithUTF8String:cname];
  NSApplication *app = [NSApplication sharedApplication];
  NSMenu *bar = [[NSMenu alloc] init];

  NSMenu *appMenu = wfSubmenu(bar, name);
  wfItem(appMenu, [@"About " stringByAppendingString:name],
         @selector(orderFrontStandardAboutPanel:), @"", 0);
  [appMenu addItem:[NSMenuItem separatorItem]];
  wfItem(appMenu, [@"Hide " stringByAppendingString:name],
         @selector(hide:), @"h", NSEventModifierFlagCommand);
  wfItem(appMenu, @"Hide Others", @selector(hideOtherApplications:), @"h",
         NSEventModifierFlagCommand | NSEventModifierFlagOption);
  wfItem(appMenu, @"Show All", @selector(unhideAllApplications:), @"", 0);
  [appMenu addItem:[NSMenuItem separatorItem]];
  wfItem(appMenu, [@"Quit " stringByAppendingString:name],
         @selector(terminate:), @"q", NSEventModifierFlagCommand);

  NSMenu *editMenu = wfSubmenu(bar, @"Edit");
  wfItem(editMenu, @"Undo", @selector(undo:), @"z", NSEventModifierFlagCommand);
  wfItem(editMenu, @"Redo", @selector(redo:), @"z",
         NSEventModifierFlagCommand | NSEventModifierFlagShift);
  [editMenu addItem:[NSMenuItem separatorItem]];
  wfItem(editMenu, @"Cut", @selector(cut:), @"x", NSEventModifierFlagCommand);
  wfItem(editMenu, @"Copy", @selector(copy:), @"c", NSEventModifierFlagCommand);
  wfItem(editMenu, @"Paste", @selector(paste:), @"v", NSEventModifierFlagCommand);
  wfItem(editMenu, @"Select All", @selector(selectAll:), @"a", NSEventModifierFlagCommand);

  NSMenu *viewMenu = wfSubmenu(bar, @"View");
  wfItem(viewMenu, @"Reload", @selector(reload:), @"r", NSEventModifierFlagCommand);

  [app setMainMenu:bar];
}

// wfSetAppName names the process. The shell is a bare binary, not a .app
// bundle, so macOS titles the app menu from the process name — "webview" —
// and ignores the menu's own title; naming the process is the only way a
// non-bundled app gets its own name up there. It must run before NSApplication
// is created, which is why it is separate from wfInstallMenu.
static void wfSetAppName(const char *cname) {
  NSString *name = [NSString stringWithUTF8String:cname];
  [[NSProcessInfo processInfo] setProcessName:name];
  // AppKit reads the app-menu title out of the main bundle's info dictionary,
  // which for a bare binary has no CFBundleName at all. The dictionary AppKit
  // hands back is mutable; seeding the key is the standard way a non-bundled
  // app names itself.
  id info = [[NSBundle mainBundle] infoDictionary];
  if ([info respondsToSelector:@selector(setObject:forKey:)]) {
    [(NSMutableDictionary *)info setObject:name forKey:@"CFBundleName"];
  }
}

// wfRaisePID activates another process's windows. The lock file records a pid,
// not a window handle, because the second launch is a different process:
// webview's own window handle is in-process only and cannot be raised from here.
static int wfRaisePID(int pid) {
  NSRunningApplication *other =
      [NSRunningApplication runningApplicationWithProcessIdentifier:(pid_t)pid];
  if (other == nil) {
    return 0;
  }
  return [other activateWithOptions:NSApplicationActivateAllWindows] ? 1 : 0;
}
*/
import "C"

import "unsafe"

// missingRuntime names what a failed window creation means on this platform.
const missingRuntime = "the system WebKit framework did not produce one"

func setAppName(name string) {
	cname := C.CString(name)
	defer C.free(unsafe.Pointer(cname))
	C.wfSetAppName(cname)
}

func installNativeMenu(name string) {
	cname := C.CString(name)
	defer C.free(unsafe.Pointer(cname))
	C.wfInstallMenu(cname)
}

// raiseInstance brings the already-running shell forward. Reporting false is
// honest failure, not an error: the caller then refuses with the running URL.
func raiseInstance(pid int) bool {
	return C.wfRaisePID(C.int(pid)) != 0
}
