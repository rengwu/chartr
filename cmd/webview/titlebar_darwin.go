//go:build webview && darwin

package main

/*
#cgo CFLAGS: -x objective-c -Wno-deprecated-declarations
#cgo LDFLAGS: -framework Cocoa

#import <Cocoa/Cocoa.h>

// WFDragView makes the cockpit's bar drag the window.
//
// It has to exist. A transparent full-size-content title bar keeps only its
// buttons interactive — every other point in the strip is passed straight down
// to the content view, by design, so that content up there stays clickable. The
// window therefore never sees the drag: the WKWebView eats it, and eats it
// whatever we put inside the web view, because WKWebView hit-tests to itself.
// This view sits over the strip, beside the web view, and hands the event back
// to the window.
//
// performWindowDragWithEvent: is the whole implementation on purpose. It is
// AppKit's own title-bar drag loop, so snapping, Spaces, window tiling and
// full-screen edges behave exactly as they do on any other window — none of
// which a hand-rolled "follow the mouse" loop would get right.
@interface WFDragView : NSView
@end

@implementation WFDragView

- (void)mouseDown:(NSEvent *)event {
  NSWindow *win = [self window];
  if (win == nil) {
    return;
  }
  if ([event clickCount] == 2) {
    // Double-click is the operator's setting, not ours: System Settings offers
    // zoom, minimise or nothing, and a title bar that ignored that would be the
    // one window on their desktop that does.
    NSString *action = [[NSUserDefaults standardUserDefaults]
        stringForKey:@"AppleActionOnDoubleClick"];
    if ([action isEqualToString:@"Minimize"]) {
      [win miniaturize:nil];
    } else if (action == nil || [action isEqualToString:@"Maximize"]) {
      [win zoom:nil];
    }
    return;
  }
  [win performWindowDragWithEvent:event];
}

// Dragging an inactive window by its bar should move it, not just focus it —
// the same as any native title bar.
- (BOOL)acceptsFirstMouse:(NSEvent *)event {
  return YES;
}

@end

// wfInstallTitleBar hands the window's top strip to the cockpit: the native
// title bar stops drawing itself and the web content extends underneath it, so
// the chrome's own bar renders there instead.
//
// The three window buttons stay native on purpose. They are AppKit's, laid out
// and lit by AppKit, and drawn *above* the content view — so re-rendering them
// in HTML would mean three fake buttons behind three real ones. Making the
// title bar transparent leaves the real ones sitting over our bar, which is the
// integration asked for, and it costs no callback into Go. They keep their
// clicks over the drag view below, because they are the one part of the
// transparent title bar AppKit still hit-tests.
//
// The returned height is the strip AppKit reserved, in points. The cockpit
// sizes its bar to exactly that, which is what centres the buttons in it: we
// never move a button, we match the height it was already centred in. Zero
// means the window could not be reshaped and the caller leaves the native title
// bar alone.
static double wfInstallTitleBar(void *ptr) {
  NSWindow *win = (NSWindow *)ptr;
  if (win == nil) {
    return 0;
  }

  [win setStyleMask:[win styleMask] | NSWindowStyleMaskFullSizeContentView];
  [win setTitlebarAppearsTransparent:YES];
  [win setTitleVisibility:NSWindowTitleHidden];
  if ([win respondsToSelector:@selector(setTitlebarSeparatorStyle:)]) {
    // Our bar draws its own bottom border on the token palette; AppKit's would
    // be a second line in a colour the design system does not own.
    [win setTitlebarSeparatorStyle:NSTitlebarSeparatorStyleNone];
  }

  // An empty unified-compact toolbar is the only supported way to ask AppKit
  // for a taller title bar, and taller is what we want: the default 28pt strip
  // is too short for the branding, and AppKit re-centres the window buttons in
  // whatever height it ends up with — on resize and full-screen too, which
  // hand-placing their frames would not survive. macOS 11+; older systems keep
  // the short strip and the bar just renders shorter.
  if ([win respondsToSelector:@selector(setToolbarStyle:)]) {
    NSToolbar *bar = [[NSToolbar alloc] initWithIdentifier:@"chartr.titlebar"];
    [bar setShowsBaselineSeparator:NO];
    [bar setAllowsUserCustomization:NO];
    [win setToolbar:bar];
    [win setToolbarStyle:NSWindowToolbarStyleUnifiedCompact];
  }

  NSView *content = [win contentView];
  if (content == nil) {
    return 0;
  }
  [content layoutSubtreeIfNeeded];
  // contentLayoutRect is the part of the content view AppKit did *not* reserve
  // for the title bar, so the difference is the strip itself.
  double h = NSHeight([content frame]) - NSHeight([win contentLayoutRect]);
  if (h < 1 || h > 200) {
    return 0;
  }

  // Where the drag view goes is the whole trick. It cannot go *inside* the
  // content view: the content view is the WKWebView, and WKWebView's hitTest:
  // answers itself for every point in its bounds so that the page gets the
  // mouse — a subview of it is never hit, whatever the order. So it goes in
  // beside it, in the window's frame view, sequenced between two siblings:
  // above the WKWebView, so the strip's drags are ours, and below the title bar
  // container, so the three window buttons still take their own clicks first.
  NSView *themeFrame = [content superview];
  if (themeFrame == nil) {
    return 0;
  }
  NSView *titlebar = [win standardWindowButton:NSWindowCloseButton];
  while (titlebar != nil && [titlebar superview] != themeFrame) {
    titlebar = [titlebar superview];
  }

  NSRect frame = [themeFrame frame];
  WFDragView *drag = [[WFDragView alloc]
      initWithFrame:NSMakeRect(0, NSHeight(frame) - h, NSWidth(frame), h)];
  [drag setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
  if (titlebar != nil) {
    [themeFrame addSubview:drag positioned:NSWindowBelow relativeTo:titlebar];
  } else {
    [themeFrame addSubview:drag positioned:NSWindowAbove relativeTo:content];
  }

  return h;
}
*/
import "C"

import (
	"math"
	"unsafe"

	webview "github.com/webview/webview_go"
)

// installTitleBar removes the native title bar and reports the height, in CSS
// pixels, of the strip the cockpit must fill in its place. Zero means the window
// keeps its native title bar and the cockpit renders no bar of its own.
func installTitleBar(w webview.WebView) int {
	h := float64(C.wfInstallTitleBar(unsafe.Pointer(w.Window())))
	if h <= 0 {
		return 0
	}
	// Points are CSS pixels on macOS whatever the backing scale, so this rounds
	// rather than converts.
	return int(math.Round(h))
}
