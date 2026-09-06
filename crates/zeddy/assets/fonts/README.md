# Terminal fonts

The terminal picker has 14 monospaced families. All fonts are embedded in the
application and available offline, with no system installation required.
`src/fonts.rs` is the shared catalog for registration and the picker.

IBM Plex Mono is the existing default. Lilex is supplied by the pinned Zed asset
bundle; `.ZedMono` is no longer offered in the terminal picker.

The additional families use unmodified static TrueType files. Regular and bold,
plus italic and bold italic where published, are included. DM Mono also includes
its light and medium faces. Static files avoid relying on variable-axis selection
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
