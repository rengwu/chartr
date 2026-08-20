//go:build webview && darwin

package main

/*
#cgo CFLAGS: -x objective-c -Wno-deprecated-declarations
#cgo LDFLAGS: -framework Cocoa -framework WebKit

#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>
#include <math.h>

static NSString *const WFPageZoomDefaultsKey = @"chartr.pageZoom.v1";
static const CGFloat WFPageZoomDefault = 1.0;
static const CGFloat WFPageZoomMinimum = 0.5;
static const CGFloat WFPageZoomMaximum = 3.0;
static const CGFloat WFPageZoomStep = 1.2;

// WFPageZoomController owns the native View-menu actions and is also the
// WKWebView's navigation delegate. Keeping those roles together makes one zoom
// value authoritative across menu validation, reloads and page notifications.
// WKWebView keeps its navigation delegate weakly, so the process-global pointer
// below deliberately retains this controller for the window's lifetime.
@interface WFPageZoomController : NSObject <NSMenuItemValidation, WKNavigationDelegate>
@property(nonatomic, assign) WKWebView *webView;
@property(nonatomic) CGFloat desiredZoom;
- (instancetype)initWithWebView:(WKWebView *)webView;
- (void)zoomIn:(id)sender;
- (void)zoomOut:(id)sender;
- (void)actualSize:(id)sender;
@end

static WFPageZoomController *gWFPageZoomController = nil;

static BOOL wfReadPageZoom(id value, CGFloat *result) {
  if (![value isKindOfClass:[NSNumber class]]) {
    return NO;
  }
  double zoom = [(NSNumber *)value doubleValue];
  if (!isfinite(zoom) || zoom < WFPageZoomMinimum ||
      zoom > WFPageZoomMaximum) {
    return NO;
  }
  *result = (CGFloat)zoom;
  return YES;
}

@implementation WFPageZoomController

- (instancetype)initWithWebView:(WKWebView *)webView {
  self = [super init];
  if (self == nil) {
    return nil;
  }

  self.webView = webView;
  NSUserDefaults *defaults = [NSUserDefaults standardUserDefaults];
  id stored = [defaults objectForKey:WFPageZoomDefaultsKey];
  CGFloat zoom = WFPageZoomDefault;
  if (stored != nil && !wfReadPageZoom(stored, &zoom)) {
    // A corrupt or obsolete preference must not make the cockpit unreadable.
    // Removing it also prevents every subsequent launch repeating the repair.
    [defaults removeObjectForKey:WFPageZoomDefaultsKey];
    zoom = WFPageZoomDefault;
  }
  self.desiredZoom = zoom;
  [self.webView setPageZoom:zoom];
  return self;
}

- (void)publishPageZoom {
  if (self.webView == nil) {
    return;
  }
  NSString *script = [NSString stringWithFormat:
      @"window.__chartrPageZoom=%.17g;window.dispatchEvent(new CustomEvent('chartr:page-zoom',{detail:%.17g}));",
      (double)self.desiredZoom, (double)self.desiredZoom];
  [self.webView evaluateJavaScript:script completionHandler:nil];
}

- (void)applyPageZoom:(CGFloat)zoom persist:(BOOL)persist publish:(BOOL)publish {
  if (!isfinite((double)zoom)) {
    zoom = WFPageZoomDefault;
  }
  zoom = MIN(WFPageZoomMaximum, MAX(WFPageZoomMinimum, zoom));
  self.desiredZoom = zoom;
  [self.webView setPageZoom:zoom];
  if (persist) {
    [[NSUserDefaults standardUserDefaults] setDouble:zoom
                                             forKey:WFPageZoomDefaultsKey];
  }
  if (publish) {
    [self publishPageZoom];
  }
}

- (void)zoomIn:(id)sender {
  [self applyPageZoom:self.desiredZoom * WFPageZoomStep
              persist:YES
              publish:YES];
}

- (void)zoomOut:(id)sender {
  [self applyPageZoom:self.desiredZoom / WFPageZoomStep
              persist:YES
              publish:YES];
}

- (void)actualSize:(id)sender {
  [self applyPageZoom:WFPageZoomDefault persist:YES publish:YES];
}

- (BOOL)validateMenuItem:(NSMenuItem *)item {
  if (self.webView == nil) {
    return NO;
  }
  SEL action = [item action];
  if (action == @selector(zoomIn:)) {
    return self.desiredZoom < WFPageZoomMaximum;
  }
  if (action == @selector(zoomOut:)) {
    return self.desiredZoom > WFPageZoomMinimum;
  }
  if (action == @selector(actualSize:)) {
    return self.desiredZoom != WFPageZoomDefault;
  }
  return YES;
}

- (void)webView:(WKWebView *)webView
    didFinishNavigation:(WKNavigation *)navigation {
  // WebKit normally carries pageZoom through a reload. Re-applying it here also
  // covers a future navigation that replaces the page, then tells that document
  // the authoritative native value after its own listeners have been installed.
  self.webView = webView;
  [self applyPageZoom:self.desiredZoom persist:NO publish:YES];
}

@end

// wfItem appends one menu item. Most callers leave the target nil so AppKit or
// WKWebView handles its standard responder-chain selector; the zoom items set
// their retained native controller as an explicit target after creation.
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
// tab had for free: Quit, whole-page zoom, Reload, and the edit items. The return
// value is the restored zoom factor so Go can seed the same value into the page
// at document start, before the navigation delegate publishes its ready event.
static double wfInstallMenu(const char *cname, void *ptr) {
  NSString *name = [NSString stringWithUTF8String:cname];
  NSWindow *win = (NSWindow *)ptr;
  WKWebView *webView = nil;
  if ([[win contentView] isKindOfClass:[WKWebView class]]) {
    webView = (WKWebView *)[win contentView];
  }

  if (gWFPageZoomController != nil) {
    [[gWFPageZoomController webView] setNavigationDelegate:nil];
    [gWFPageZoomController release];
  }
  gWFPageZoomController =
      [[WFPageZoomController alloc] initWithWebView:webView];
  [webView setNavigationDelegate:gWFPageZoomController];

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
  NSMenuItem *zoomIn = wfItem(viewMenu, @"Zoom In", @selector(zoomIn:), @"+",
                              NSEventModifierFlagCommand);
  [zoomIn setTarget:gWFPageZoomController];
  NSMenuItem *zoomOut = wfItem(viewMenu, @"Zoom Out", @selector(zoomOut:), @"-",
                               NSEventModifierFlagCommand);
  [zoomOut setTarget:gWFPageZoomController];
  NSMenuItem *actualSize = wfItem(viewMenu, @"Actual Size",
                                  @selector(actualSize:), @"0",
                                  NSEventModifierFlagCommand);
  [actualSize setTarget:gWFPageZoomController];
  [viewMenu addItem:[NSMenuItem separatorItem]];
  wfItem(viewMenu, @"Reload", @selector(reload:), @"r", NSEventModifierFlagCommand);

  [app setMainMenu:bar];
  return (double)[gWFPageZoomController desiredZoom];
}

// wfSetAppName names the process. Launched loose the shell is a bare binary with
// no .app bundle, so macOS titles the app menu from the process name — "webview"
// — and ignores the menu's own title; naming the process is the only way that
// build gets its own name up there. It must run before NSApplication is created,
// which is why it is separate from wfInstallMenu.
//
// Inside the chartr.app `make bundle` assembles (ADR 0016) the property list
// already carries CFBundleName, so this writes the same name a second time and
// changes nothing. The loose shell is still a shipped artifact, so it stays.
static void wfSetAppName(const char *cname) {
  NSString *name = [NSString stringWithUTF8String:cname];
  [[NSProcessInfo processInfo] setProcessName:name];
  // AppKit reads the app-menu title out of the main bundle's info dictionary,
  // which for a bare binary has no CFBundleName at all (a bundled launch reads
  // its own, which already says chartr). The dictionary AppKit hands back is
  // mutable; seeding the key is the standard way a non-bundled app names itself.
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

func installNativeMenu(name string, window unsafe.Pointer) float64 {
	cname := C.CString(name)
	defer C.free(unsafe.Pointer(cname))
	return float64(C.wfInstallMenu(cname, window))
}

// raiseInstance brings the already-running shell forward. Reporting false is
// honest failure, not an error: the caller then refuses with the running URL.
func raiseInstance(pid int) bool {
	return C.wfRaisePID(C.int(pid)) != 0
}
