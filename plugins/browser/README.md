# chartr Browser

chartr's first-party, separately installed browser pane. Each pane owns one web
page; chartr's normal tabs and splits provide multi-page layouts.

This repository is intentionally a tiny hosted plugin package. The manifest
activates chartr's built-in `browser` surface, which uses the operating-system
web engine (WebKit on macOS and WebKitGTK on Linux). Keeping the window and
webview lifecycle inside chartr avoids loading a second copy of GPUI from a
plugin library.

There is no build or platform artifact. Install directly from this Git
repository, or choose this folder under **Settings → Plugins → Install from
Folder…**. chartr validates and copies the package, then offers **Restart** or
**Later**.

The Browser intentionally has no devtools, browser tab strip, bookmarks,
history UI, permission manager, or download manager. Web-engine storage is
ephemeral; the current URL is the only Browser state retained across launches.
