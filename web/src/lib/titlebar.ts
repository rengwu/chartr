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
    __chartrSetTitleBarButtonRects?: (rects: TitleBarButtonRect[]) => unknown;
  }
}

export interface TitleBarButtonRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function nativeTitleBarHeight(): number {
  const h = typeof window === "undefined" ? undefined : window.__chartrTitleBar;
  return typeof h === "number" && h > 0 ? h : 0;
}

// Keep the native drag overlay's passthrough regions exactly aligned with the
// clickable controls currently occupying the top strip. The generic selector is
// intentional: settings and future header variants inherit correct dragging
// without maintaining a second, native list of which buttons each route renders.
export function trackTitleBarButtons(height: number): () => void {
  const report = window.__chartrSetTitleBarButtonRects;
  if (height <= 0 || typeof report !== "function") return () => {};

  let frame = 0;
  const update = () => {
    if (frame !== 0) cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      frame = 0;
      const rects = Array.from(
        document.querySelectorAll<HTMLElement>(
          "button, a[href], input, select, textarea, [role='button']",
        ),
      )
        .filter((element) => {
          const style = getComputedStyle(element);
          if (style.display === "none" || style.visibility === "hidden" || style.pointerEvents === "none") {
            return false;
          }
          const rect = element.getBoundingClientRect();
          return rect.width > 0 && rect.height > 0 && rect.top < height && rect.bottom > 0;
        })
        .map((element): TitleBarButtonRect => {
          const rect = element.getBoundingClientRect();
          const top = Math.max(0, rect.top);
          const bottom = Math.min(height, rect.bottom);
          return {
            x: rect.left,
            y: top,
            width: rect.width,
            height: bottom - top,
          };
        });
      void Promise.resolve(report(rects)).catch(() => {});
    });
  };

  // Seat the initial rectangles before attaching optional observation APIs. An
  // older WKWebView without ResizeObserver must still get working header buttons.
  update();
  const mutations = new MutationObserver(update);
  mutations.observe(document.body, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["class", "style", "hidden", "aria-hidden"],
  });
  const resize = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(update);
  resize?.observe(document.documentElement);
  window.addEventListener("resize", update);

  return () => {
    if (frame !== 0) cancelAnimationFrame(frame);
    mutations.disconnect();
    resize?.disconnect();
    window.removeEventListener("resize", update);
    void Promise.resolve(report([])).catch(() => {});
  };
}

// Whether the window draws its own native title bar above the page — the shell
// on Linux and Windows, where the OS keeps the top strip and we never got one to
// draw in. The two halves of the test are both needed: the platform marker is
// what separates the shell from a plain browser tab (whose chrome is the
// browser's, not the window's), and a zero strip height is what separates those
// shells from macOS, where the shell hands us the strip instead.
//
// The chrome uses this to drop its wordmark: under a native title bar the window
// is already named by the OS, so repeating the brand inside the sidebar only
// pushes the search field a tier down for nothing.
export function hasNativeTitleBar(): boolean {
  if (typeof window === "undefined") return false;
  const platform = window.__chartrNativePlatform;
  return typeof platform === "string" && platform !== "" && nativeTitleBarHeight() === 0;
}
