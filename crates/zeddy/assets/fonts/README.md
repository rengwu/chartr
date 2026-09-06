# Interface fonts

The interface picker offers IBM Plex Sans, System UI, and 23 additional Google
Fonts families. `.ZedSans` is no longer offered; saved Zed Sans preferences
resolve to IBM Plex Sans. IBM Plex Sans remains supplied by Zed's asset bundle.

The added families are Asap, Barlow, Cabin, Comic Neue, DM Sans, Fira Sans, Geist,
Hind, Inter, Jim Nightshade, Karla, Lato, Merriweather Sans, Noto Sans, Nunito Sans,
Open Sans, Oxygen, Playpen Sans, Poppins, PT Sans, PT Serif, Roboto, and Work Sans.
All are embedded and available offline without system installation.
`src/fonts.rs` supplies both the picker choices and font registration.

`ui/` contains unmodified static TrueType files served by the Google Fonts CSS
API, with regular, bold, italic, and bold italic faces where available. Geist,
Hind, Oxygen, and Playpen Sans include upright faces; Jim Nightshade publishes
only regular. Static faces avoid variable-axis selection in the native
font backend. Internal family names and weight metadata were verified against
the catalog, and the 81 files add approximately 3.83 MiB before packaging.
DM Sans and Nunito Sans retain optical-size names internally; `native_family`
in `src/fonts.rs` maps their picker names to those exact names without modifying
the publisher's files. The manifest records these names as `native_family`.

Each family includes its SIL Open Font License notice from Google Fonts revision
`5e35378e6bda803962ee6fd257e444a7d459660d`. [ui/sources.json](ui/sources.json)
records the exact versioned download URLs, SHA-256 checksums, weights, styles,
CSS requests, and pinned license sources. To reproduce the assets, download each
face's `url` to its `file` path under `ui/` and verify its `sha256` checksum.

# Terminal fonts

The terminal picker has 14 monospaced families. All fonts are embedded in the
application and available offline, with no system installation required.
`src/fonts.rs` is the shared catalog for registration and the picker.

IBM Plex Mono is the existing default. Lilex is supplied by the pinned Zed asset
bundle; `.ZedMono` is no longer offered in the terminal picker.

The additional families use unmodified static TrueType files. Regular and bold,
plus italic and bold italic where published, are included. DM Mono retains its
medium faces as the closest available weight for bold terminal text; unused
light faces are excluded. Static files avoid relying on variable-axis selection
in the native font backend. Cutive Mono and PT Mono only publish a regular face;
Fira Code and Inconsolata do not publish italics in these sources.

Each directory contains its publisher's SIL Open Font License 1.1 notice.
Lilex's license stays with Zed's assets.

## Sources

| Family | Pinned source |
| --- | --- |
| IBM Plex Mono | Existing `ibm-plex-mono/` asset and `LICENSE.txt` |
| Lilex | Zed `1ea16c1ab9dd6d36649e002dc60995634da04daf`, `assets/fonts/lilex/` |
| Anonymous Pro | [Google Fonts](https://github.com/google/fonts/tree/5e35378e6bda803962ee6fd257e444a7d459660d/ofl/anonymouspro) |
| Cousine | [Google Fonts](https://github.com/google/fonts/tree/5e35378e6bda803962ee6fd257e444a7d459660d/ofl/cousine) |
| Cutive Mono | [Google Fonts](https://github.com/google/fonts/tree/5e35378e6bda803962ee6fd257e444a7d459660d/ofl/cutivemono) |
| DM Mono | [Google Fonts](https://github.com/google/fonts/tree/5e35378e6bda803962ee6fd257e444a7d459660d/ofl/dmmono) |
| PT Mono | [Google Fonts](https://github.com/google/fonts/tree/5e35378e6bda803962ee6fd257e444a7d459660d/ofl/ptmono) |
| Space Mono | [Google Fonts](https://github.com/google/fonts/tree/5e35378e6bda803962ee6fd257e444a7d459660d/ofl/spacemono) |
| Fira Code | [Publisher release 6.2](https://github.com/tonsky/FiraCode/releases/tag/6.2), `ttf/` |
| Inconsolata | [Publisher repository](https://github.com/googlefonts/Inconsolata/tree/fc1fc21081558b39a2db43bfd9b65bf9acb50701/fonts/ttf) |
| JetBrains Mono | [Publisher repository](https://github.com/JetBrains/JetBrainsMono/tree/19371302b95d218af43299bce79ddbddd0bc364d/fonts/ttf) |
| Red Hat Mono | [Publisher repository](https://github.com/RedHatOfficial/RedHatFont/tree/6bb1048a6402b0076ea04f42951ec66263cd1437/fonts/Mono/RedHatMono/ttf) |
| Roboto Mono | [Publisher repository](https://github.com/googlefonts/RobotoMono/tree/895ec691990d041dd727c7b5afa3ce56525d98e6/fonts/ttf) |
| Source Code Pro | [Publisher repository](https://github.com/adobe-fonts/source-code-pro/tree/803b7e23ec97ae58b6232ea76519a76d428ba268/TTF) |
