# chartr Browser

chartr's bundled browser pane. Each pane owns one web
page; chartr's normal tabs and splits provide multi-page layouts.

This repository is intentionally a tiny hosted plugin package. The manifest
activates chartr's built-in `browser` surface, which uses the operating-system
web engine (WebKit on macOS and WebKitGTK on Linux). Keeping the window and
webview lifecycle inside chartr avoids loading a second copy of GPUI from a
plugin library.

The manifest and icon ship with chartr; open **Browser** from the workspace’s
**New surface** menu. No separate installation is needed. The package can still
be installed from this folder as an override through **Settings → Plugins →
Install from Folder…**. Saved disable and uninstall preferences are respected.

The Browser intentionally has no devtools, browser tab strip, bookmarks,
history UI, permission manager, or download manager. Web-engine storage is
ephemeral; the current URL is the only Browser state retained across launches.
