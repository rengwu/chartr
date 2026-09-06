export type Appearance = 'Dark' | 'Light'

export type ThemeTokens = {
  surface: string
  sidebar: string
  border: string
  text: string
  muted: string
  card: string
  cardOpen: string
  ring: string
  selected: string
  hover: string
  notice: string
  accent: string
  done: string
  idle: string
  quiet: string
  terminalForeground: string
  sidebarCardInactive: string
  sidebarCardActive: string
  sidebarSessionHover: string
  sidebarSessionActive: string
}

export type ThemePreset = {
  name: string
  appearance: Appearance
  tokens: ThemeTokens
}

type BaseTokens = Omit<
  ThemeTokens,
  'sidebarCardInactive' | 'sidebarCardActive' | 'sidebarSessionHover' | 'sidebarSessionActive'
>

const tokens = (base: BaseTokens, sidebar: [string, string, string, string]): ThemeTokens => ({
  ...base,
  sidebarCardInactive: sidebar[0],
  sidebarCardActive: sidebar[1],
  sidebarSessionHover: sidebar[2],
  sidebarSessionActive: sidebar[3],
})

// Sourced from crates/zeddy/src/settings.rs. Keep this catalog in the same order
// as THEME_PALETTES, followed by Chartr's derived dark/light themes.
export const THEME_PRESETS: ThemePreset[] = [
  {
    name: 'Ayu Dark', appearance: 'Dark',
    tokens: tokens({ surface: '0x0d1016', sidebar: '0x131820', border: '0x36404e', text: '0xd8d5cd', muted: '0xa9adb5', card: '0x202731', cardOpen: '0x374352', ring: '0xe6b450', selected: '0x374352', hover: '0x2c3541', notice: '0xf08080', accent: '0xe6b450', done: '0xaacb73', idle: '0xe6b450', quiet: '0x7d8795', terminalForeground: '0xd8d5cd' }, ['0x1d242e', '0x242e3a', '0x2a3542', '0x313e4e']),
  },
  {
    name: 'Ayu Light', appearance: 'Light',
    tokens: tokens({ surface: '0xfcfcfa', sidebar: '0xeceef0', border: '0xbfc5cb', text: '0x343b43', muted: '0x59636e', card: '0xf5f6f7', cardOpen: '0xdce2e7', ring: '0xa66316', selected: '0xdce2e7', hover: '0xe5e9ed', notice: '0xc44545', accent: '0xa66316', done: '0x527c23', idle: '0x936115', quiet: '0x717b86', terminalForeground: '0x343b43' }, ['0xfafbfc', '0xf3f5f7', '0xe5e9ed', '0xd5dde5']),
  },
  {
    name: 'Ayu Mirage', appearance: 'Dark',
    tokens: tokens({ surface: '0x242936', sidebar: '0x1c212c', border: '0x475266', text: '0xd9d7ce', muted: '0xb0b7c4', card: '0x2b3342', cardOpen: '0x465369', ring: '0xeac080', selected: '0x465369', hover: '0x354154', notice: '0xef9385', accent: '0xeac080', done: '0xb9d68a', idle: '0xeac080', quiet: '0x8995a8', terminalForeground: '0xd9d7ce' }, ['0x293141', '0x303b4d', '0x374357', '0x404e64']),
  },
  {
    name: 'Catppuccin Frappé', appearance: 'Dark',
    tokens: tokens({ surface: '0x303446', sidebar: '0x292c3c', border: '0x51576d', text: '0xc6d0f5', muted: '0xa5adce', card: '0x414559', cardOpen: '0x51576d', ring: '0xca9ee6', selected: '0x51576d', hover: '0x414559', notice: '0xe78284', accent: '0xca9ee6', done: '0xa6d189', idle: '0xe5c890', quiet: '0x737994', terminalForeground: '0xc6d0f5' }, ['0x2f3243', '0x35394b', '0x373b4d', '0x414559']),
  },
  {
    name: 'Catppuccin Latte', appearance: 'Light',
    tokens: tokens({ surface: '0xeff1f5', sidebar: '0xe6e9ef', border: '0xbcc0cc', text: '0x4c4f69', muted: '0x606379', card: '0xe9eaf1', cardOpen: '0xd8d5e8', ring: '0x8839ef', selected: '0xd8d5e8', hover: '0xe3e1ee', notice: '0xd20f39', accent: '0x8839ef', done: '0x397d27', idle: '0x94620f', quiet: '0x74788d', terminalForeground: '0x4c4f69' }, ['0xf5f6fa', '0xeef0f6', '0xe2e2ed', '0xd9d5e9']),
  },
  {
    name: 'Catppuccin Macchiato', appearance: 'Dark',
    tokens: tokens({ surface: '0x24273a', sidebar: '0x1e2030', border: '0x494d64', text: '0xcad3f5', muted: '0xa5adcb', card: '0x363a4f', cardOpen: '0x494d64', ring: '0xc6a0f6', selected: '0x494d64', hover: '0x363a4f', notice: '0xed8796', accent: '0xc6a0f6', done: '0xa6da95', idle: '0xeed49f', quiet: '0x6e738d', terminalForeground: '0xcad3f5' }, ['0x242738', '0x2a2d40', '0x2c3043', '0x363a4f']),
  },
  {
    name: 'Catppuccin Mocha', appearance: 'Dark',
    tokens: tokens({ surface: '0x1e1e2e', sidebar: '0x181825', border: '0x45475a', text: '0xcdd6f4', muted: '0xa6adc8', card: '0x313244', cardOpen: '0x45475a', ring: '0xcba6f7', selected: '0x45475a', hover: '0x313244', notice: '0xf38ba8', accent: '0xcba6f7', done: '0xa6e3a1', idle: '0xf9e2af', quiet: '0x6c7086', terminalForeground: '0xcdd6f4' }, ['0x1e1f2d', '0x252535', '0x272838', '0x313244']),
  },
  {
    name: 'Gruvbox Dark', appearance: 'Dark',
    tokens: tokens({ surface: '0x242321', sidebar: '0x1b1b1a', border: '0x494740', text: '0xe3dccb', muted: '0xb6afa0', card: '0x2b2a27', cardOpen: '0x46443c', ring: '0xd8a657', selected: '0x46443c', hover: '0x35342f', notice: '0xe0786c', accent: '0xd8a657', done: '0xa9b783', idle: '0xd8a657', quiet: '0x8c887c', terminalForeground: '0xe3dccb' }, ['0x282724', '0x302f2a', '0x37362f', '0x403e36']),
  },
  {
    name: 'Gruvbox Light', appearance: 'Light',
    tokens: tokens({ surface: '0xfbf1c7', sidebar: '0xecddb4', border: '0xc8b899', text: '0x282828', muted: '0x5f5650', card: '0xecddb4', cardOpen: '0xc8b899', ring: '0xab9965', selected: '0xc8b899', hover: '0xddcca7', notice: '0x9d0308', accent: '0x0b6678', done: '0x797410', idle: '0xb57615', quiet: '0x897b6e', terminalForeground: '0x282828' }, ['0xf0e6c9', '0xf0e6c9', '0xe3d3ac', '0xddcca7']),
  },
  {
    name: 'One Dark', appearance: 'Dark',
    tokens: tokens({ surface: '0x242730', sidebar: '0x1c1f27', border: '0x454b5b', text: '0xdce1ed', muted: '0xacb5c8', card: '0x2b303c', cardOpen: '0x434c60', ring: '0xb5a0e8', selected: '0x434c60', hover: '0x343c4c', notice: '0xe58a94', accent: '0xb5a0e8', done: '0xa2c98c', idle: '0xe5c286', quiet: '0x8290a6', terminalForeground: '0xd0d7e5' }, ['0x282e3a', '0x303949', '0x374356', '0x3e4c62']),
  },
  {
    name: 'One Light', appearance: 'Light',
    tokens: tokens({ surface: '0xfafbfc', sidebar: '0xe9edf2', border: '0xbcc4d0', text: '0x2c313c', muted: '0x555f70', card: '0xf0f2f6', cardOpen: '0xd7deeb', ring: '0x5264ba', selected: '0xd7deeb', hover: '0xe3e8f1', notice: '0xbd4c49', accent: '0x5264ba', done: '0x467b3f', idle: '0x866915', quiet: '0x727d90', terminalForeground: '0x2c313c' }, ['0xf9fafc', '0xf0f3f8', '0xe3e8f1', '0xd3dced']),
  },
  {
    name: 'VSCode Dark Modern', appearance: 'Dark',
    tokens: tokens({ surface: '0x1f1f1f', sidebar: '0x181818', border: '0x2b2b2b', text: '0xcccccc', muted: '0x9d9d9d', card: '0x313131', cardOpen: '0x313131', ring: '0x0078d4', selected: '0x313131', hover: '0x2b2b2b', notice: '0xf85149', accent: '0x0078d4', done: '0x2ea043', idle: '0xe2c08d', quiet: '0x6e7681', terminalForeground: '0xcccccc' }, ['0x1d1d1d', '0x222222', '0x232323', '0x2b2b2b']),
  },
  {
    name: 'VSCode Dark Plus', appearance: 'Dark',
    tokens: tokens({ surface: '0x1e1e1e', sidebar: '0x252526', border: '0x3f3f46', text: '0xd4d4d4', muted: '0x969696', card: '0x2d2d30', cardOpen: '0x37373d', ring: '0x007acc', selected: '0x37373d', hover: '0x2a2d2e', notice: '0xf44747', accent: '0x007acc', done: '0x6a9955', idle: '0xdcdcaa', quiet: '0x707070', terminalForeground: '0xd4d4d4' }, ['0x2c2c2f', '0x323235', '0x38383c', '0x3f3f44']),
  },
  {
    name: 'Chartr Dark', appearance: 'Dark',
    tokens: tokens({ surface: '0x282c33', sidebar: '0x2f343e', border: '0x505866', text: '0xdce0e5', muted: '0xa9afbc', card: '0x2e343e', cardOpen: '0x454a56', ring: '0x47679e', selected: '0x454a56', hover: '0x363c46', notice: '0xd07277', accent: '0x74ade8', done: '0xa1c181', idle: '0xdec184', quiet: '0x878a98', terminalForeground: '0xabb2bf' }, ['0x353c47', '0x39414e', '0x3e4857', '0x444f61']),
  },
  {
    name: 'Chartr Light', appearance: 'Light',
    tokens: tokens({ surface: '0xf7f8fa', sidebar: '0xffffff', border: '0xbcc6d4', text: '0x24272d', muted: '0x505d70', card: '0xeef1f5', cardOpen: '0xd6dfed', ring: '0x4263c7', selected: '0xd6dfed', hover: '0xe3e8f0', notice: '0xbd4c49', accent: '0x4263c7', done: '0x467b3f', idle: '0x866915', quiet: '0x505d70', terminalForeground: '0x24272d' }, ['0xedf0f4', '0xe5eaf2', '0xd6deea', '0xc6d3e6']),
  },
]

export type TokenKey = keyof ThemeTokens

export type TokenGroup = {
  label: string
  description: string
  tokens: { key: TokenKey; label: string; hint: string }[]
}

export const TOKEN_GROUPS: TokenGroup[] = [
  {
    label: 'Foundations', description: 'Window layers and boundaries',
    tokens: [
      { key: 'surface', label: 'Surface', hint: 'Canvas, editor, terminal' },
      { key: 'sidebar', label: 'Sidebar', hint: 'Navigation and panel base' },
      { key: 'border', label: 'Border', hint: 'Dividers and outlines' },
      { key: 'card', label: 'Card', hint: 'Resting controls and cards' },
      { key: 'cardOpen', label: 'Card open', hint: 'Active and open cards' },
    ],
  },
  {
    label: 'Content', description: 'Text, icons, and terminal output',
    tokens: [
      { key: 'text', label: 'Text', hint: 'Primary labels and icons' },
      { key: 'muted', label: 'Muted', hint: 'Secondary information' },
      { key: 'quiet', label: 'Quiet', hint: 'Disabled and hidden items' },
      { key: 'terminalForeground', label: 'Terminal', hint: 'Terminal foreground' },
    ],
  },
  {
    label: 'Interaction', description: 'Hover, selection, and focus',
    tokens: [
      { key: 'hover', label: 'Hover', hint: 'Hover state' },
      { key: 'selected', label: 'Selected', hint: 'Selected background' },
      { key: 'ring', label: 'Focus ring', hint: 'Focus and drop targets' },
      { key: 'accent', label: 'Accent', hint: 'Links and information' },
    ],
  },
  {
    label: 'Status', description: 'Semantic system states',
    tokens: [
      { key: 'notice', label: 'Notice', hint: 'Error and deleted' },
      { key: 'done', label: 'Done', hint: 'Success and added' },
      { key: 'idle', label: 'Idle', hint: 'Warning and modified' },
    ],
  },
  {
    label: 'Chartr sidebar', description: 'App-specific sidebar layering',
    tokens: [
      { key: 'sidebarCardInactive', label: 'Card inactive', hint: 'Inactive space card' },
      { key: 'sidebarCardActive', label: 'Card active', hint: 'Active space card' },
      { key: 'sidebarSessionHover', label: 'Session hover', hint: 'Hovered session row' },
      { key: 'sidebarSessionActive', label: 'Session active', hint: 'Active session row' },
    ],
  },
]

export const clonePreset = (preset: ThemePreset): ThemePreset => ({
  ...preset,
  tokens: { ...preset.tokens },
})

export const normalizeRustHex = (value: string, fallback = '0x000000') => {
  const clean = value.trim().replace(/^#|^0x/i, '').replace(/[^0-9a-f]/gi, '').slice(0, 6)
  return clean.length === 6 ? `0x${clean.toLowerCase()}` : fallback
}

export const toCssHex = (value: string) => `#${normalizeRustHex(value).slice(2)}`

export const makeRustExport = (theme: ThemePreset) => {
  const t = theme.tokens
  const values = [t.surface, t.sidebar, t.border, t.text, t.muted, t.card, t.cardOpen, t.ring, t.selected, t.hover, t.notice, t.accent, t.done, t.idle, t.quiet, t.terminalForeground]
  const themeLines = values.map((value) => `    ${normalizeRustHex(value)},`).join('\n')
  return `// Generated by misc/theme-playground
// 1. Bump THEME_PALETTES' array length and insert this entry.
ThemePalette::new(
    ${JSON.stringify(theme.name)},
    Appearance::${theme.appearance},
${themeLines}
),

// 2. Bump SIDEBAR_THEME_PALETTES' array length and insert this entry.
SidebarThemePalette::new(
    ${JSON.stringify(theme.name)},
    ${normalizeRustHex(t.sidebarCardInactive)},
    ${normalizeRustHex(t.sidebarCardActive)},
    ${normalizeRustHex(t.sidebarSessionHover)},
    ${normalizeRustHex(t.sidebarSessionActive)},
),`
}
