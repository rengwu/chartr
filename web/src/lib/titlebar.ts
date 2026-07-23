// The native shell's custom title bar (macOS only).
//
// The shell strips the window's native title bar and injects the height of the
// strip it freed, in CSS pixels, before the document loads. That injection is
// the whole contract: its presence means "you are in a window whose top strip is
// yours to draw", and its value is the height that keeps the three native window
// buttons — still AppKit's, drawn above the page — centred in our bar.
//
// A plain browser tab and the non-macOS shells never see the global, so they get
// zero and render no bar at all.
declare global {
  interface Window {
    __chartrTitleBar?: number;
  }
}

export function nativeTitleBarHeight(): number {
  const h = typeof window === "undefined" ? undefined : window.__chartrTitleBar;
  return typeof h === "number" && h > 0 ? h : 0;
}
