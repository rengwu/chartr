// The seam tests run the island headless — no 2D context, nothing drawn. jsdom
// has no canvas backend and logs a "not implemented" error on getContext; stub
// it to quietly return null (the island's own graceful-degradation path) so the
// test output stays clean.
HTMLCanvasElement.prototype.getContext = (() => null) as never
