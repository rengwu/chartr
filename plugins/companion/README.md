# Companion

The bundled `com.chartr.companion` plugin connects [Chartr Mobile](../../../chartr-mobile) to the terminal sessions in its desktop window.

## Connect

Open **New surface → Companion**, choose the bind address, and press **Start sharing**. The default `0.0.0.0:9847` listens on IPv4 interfaces. Enter the computer’s reachable LAN or Tailscale address and port in the app. This development build accepts all connections without a pairing code or authentication. TLS encrypts traffic; the phone does not verify the server certificate.

Sharing remembers its enabled state and address in the plugin’s data directory and resumes after a restart. **Stop sharing** saves the disabled state. Disabling/unloading the plugin or closing the window stops its listener and active connections.

## Terminal ownership

Opening a mobile terminal claims its geometry for that connection. Columns and rows follow the phone’s available viewport at a readable font size, including keyboard and rotation changes. Desktop displays a read-only mirror while mobile owns the terminal, so desktop layout cannot fight mobile resizes. Leaving the terminal, disconnecting, or an expired lease restores desktop sizing. Another mobile connection can browse but cannot steal an active viewer’s sizing or send input to that owned terminal.

Mobile **History** reads Herdr’s complete retained buffer through `pane.selection.read`, not the attach client’s viewport repaint or the capped `pane.read` result. Immutable UTF-8 pages preserve blank lines and large histories. The reader freezes old output and holds a logical text anchor while scrolled away from the bottom; **Latest** resumes following.

The desktop remains the only Herdr attachment. Mobile never spawns a competing takeover client. Terminal commands use the existing PTY; Send submits paste and Enter in one host operation.

## Verification

See [the protocol](../../docs/companion-protocol.md). Transport tests cover open-access TLS, framing, request ordering, separate connection lifetimes, and revocation. GPUI tests cover plugin shutdown and saved sharing state. Android instrumentation exercises the real host and PTY.

For isolated tests, build with `companion-test-host`, set `CHARTR_COMPANION_TEST_READY` to a new file under an absolute directory named `chartr-companion-test`, and place all three XDG config/data/state roots beneath it. The fixture listens on `127.0.0.1:19847`. It is excluded from normal builds.
