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
    tokens: tokens({ surface: '0x0d1016', sidebar: '0x1f2127', border: '0x3f4043', text: '0xbfbdb6', muted: '0x8a8986', card: '0x1f2127', cardOpen: '0x3e4043', ring: '0x1b4a6e', selected: '0x3e4043', hover: '0x2d2f34', notice: '0xef7177', accent: '0x5ac1fe', done: '0xaad84c', idle: '0xfeb454', quiet: '0x696a6a', terminalForeground: '0xbfbdb6' }, ['0x23252a', '0x26282e', '0x27292f', '0x2d2f34']),
  },
  {
    name: 'Ayu Light', appearance: 'Light',
    tokens: tokens({ surface: '0xfcfcfc', sidebar: '0xececed', border: '0xcfd1d2', text: '0x5c6166', muted: '0x8b8e92', card: '0xececed', cardOpen: '0xcfd0d2', ring: '0xc4daf6', selected: '0xcfd0d2', hover: '0xdfe0e1', notice: '0xef7271', accent: '0x3b9ee5', done: '0x85b304', idle: '0xf1ad49', quiet: '0xa9acae', terminalForeground: '0x5c6166' }, ['0xe9e9ea', '0xe6e6e7', '0xe4e5e6', '0xdfe0e1']),
  },
  {
    name: 'Ayu Mirage', appearance: 'Dark',
    tokens: tokens({ surface: '0x242835', sidebar: '0x353944', border: '0x53565d', text: '0xcccac2', muted: '0x9a9a98', card: '0x353944', cardOpen: '0x53565d', ring: '0x24556f', selected: '0x53565d', hover: '0x43464f', notice: '0xf18779', accent: '0x72cffe', done: '0xd5fe80', idle: '0xfecf72', quiet: '0x7b7d7f', terminalForeground: '0xcccac2' }, ['0x393c47', '0x3c404a', '0x3d414b', '0x43464f']),
  },
  {
    name: 'Catppuccin Frappé', appearance: 'Dark',
    tokens: tokens({ surface: '0x303446', sidebar: '0x292c3c', border: '0x51576d', text: '0xc6d0f5', muted: '0xa5adce', card: '0x414559', cardOpen: '0x51576d', ring: '0xca9ee6', selected: '0x51576d', hover: '0x414559', notice: '0xe78284', accent: '0xca9ee6', done: '0xa6d189', idle: '0xe5c890', quiet: '0x737994', terminalForeground: '0xc6d0f5' }, ['0x2f3243', '0x35394b', '0x373b4d', '0x414559']),
  },
  {
    name: 'Catppuccin Latte', appearance: 'Light',
    tokens: tokens({ surface: '0xeff1f5', sidebar: '0xe6e9ef', border: '0xbcc0cc', text: '0x4c4f69', muted: '0x6c6f85', card: '0xccd0da', cardOpen: '0xbcc0cc', ring: '0x8839ef', selected: '0xbcc0cc', hover: '0xccd0da', notice: '0xd20f39', accent: '0x8839ef', done: '0x40a02b', idle: '0xdf8e1d', quiet: '0x9ca0b0', terminalForeground: '0x4c4f69' }, ['0xe0e3ea', '0xd9dde5', '0xd6dae2', '0xccd0da']),
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
    tokens: tokens({ surface: '0x282828', sidebar: '0x3a3735', border: '0x5b534d', text: '0xfbf1c7', muted: '0xc5b597', card: '0x3a3735', cardOpen: '0x5b524c', ring: '0x303a36', selected: '0x5b524c', hover: '0x494340', notice: '0xfb4a35', accent: '0x83a598', done: '0xb7bb26', idle: '0xf9bd2f', quiet: '0x998b78', terminalForeground: '0xebdbb2' }, ['0x3e3a38', '0x423d3b', '0x433e3c', '0x494340']),
  },
  {
    name: 'Gruvbox Light', appearance: 'Light',
    tokens: tokens({ surface: '0xfbf1c7', sidebar: '0xecddb4', border: '0xc8b899', text: '0x282828', muted: '0x5f5650', card: '0xecddb4', cardOpen: '0xc8b899', ring: '0xab9965', selected: '0xc8b899', hover: '0xddcca7', notice: '0x9d0308', accent: '0x0b6678', done: '0x797410', idle: '0xb57615', quiet: '0x897b6e', terminalForeground: '0x282828' }, ['0xf0e6c9', '0xf0e6c9', '0xe3d3ac', '0xddcca7']),
  },
  {
    name: 'One Dark', appearance: 'Dark',
    tokens: tokens({ surface: '0x282c33', sidebar: '0x2f343e', border: '0x464b57', text: '0xdce0e5', muted: '0xa9afbc', card: '0x2e343e', cardOpen: '0x454a56', ring: '0x47679e', selected: '0x454a56', hover: '0x363c46', notice: '0xd07277', accent: '0x74ade8', done: '0xa1c181', idle: '0xdec184', quiet: '0x878a98', terminalForeground: '0xabb2bf' }, ['0x313640', '0x333842', '0x333943', '0x363c46']),
  },
  {
    name: 'One Light', appearance: 'Light',
    tokens: tokens({ surface: '0xfafafa', sidebar: '0xebebec', border: '0xc9c9ca', text: '0x242529', muted: '0x58585a', card: '0xebebec', cardOpen: '0xcacaca', ring: '0x7d82e8', selected: '0xcacaca', hover: '0xdfdfe0', notice: '0xd36151', accent: '0x5c78e2', done: '0x669f59', idle: '0xa48819', quiet: '0x7e8086', terminalForeground: '0x2a2c33' }, ['0xe8e8e9', '0xe5e5e6', '0xe4e4e5', '0xdfdfe0']),
  },
  {
    name: 'VSCode Dark Modern', appearance: 'Dark',
    tokens: tokens({ surface: '0x1f1f1f', sidebar: '0x181818', border: '0x2b2b2b', text: '0xcccccc', muted: '0x9d9d9d', card: '0x313131', cardOpen: '0x313131', ring: '0x0078d4', selected: '0x313131', hover: '0x2b2b2b', notice: '0xf85149', accent: '0x0078d4', done: '0x2ea043', idle: '0xe2c08d', quiet: '0x6e7681', terminalForeground: '0xcccccc' }, ['0x1d1d1d', '0x222222', '0x232323', '0x2b2b2b']),
  },
  {
    name: 'VSCode Dark Plus', appearance: 'Dark',
    tokens: tokens({ surface: '0x1e1e1e', sidebar: '0x252526', border: '0x3f3f46', text: '0xd4d4d4', muted: '0x969696', card: '0x2d2d30', cardOpen: '0x37373d', ring: '0x007acc', selected: '0x37373d', hover: '0x2a2d2e', notice: '0xf44747', accent: '0x007acc', done: '0x6a9955', idle: '0xdcdcaa', quiet: '0x707070', terminalForeground: '0xd4d4d4' }, ['0x262728', '0x28292a', '0x282a2b', '0x2a2d2e']),
  },
  {
    name: 'Chartr Dark', appearance: 'Dark',
    tokens: tokens({ surface: '0x282c33', sidebar: '0x2f343e', border: '0x505866', text: '0xdce0e5', muted: '0xa9afbc', card: '0x2e343e', cardOpen: '0x454a56', ring: '0x47679e', selected: '0x454a56', hover: '0x363c46', notice: '0xd07277', accent: '0x74ade8', done: '0xa1c181', idle: '0xdec184', quiet: '0x878a98', terminalForeground: '0xabb2bf' }, ['0x313640', '0x333842', '0x333943', '0x363c46']),
  },
  {
    name: 'Chartr Light', appearance: 'Light',
    tokens: tokens({ surface: '0xf7f8fa', sidebar: '0xffffff', border: '0xd4d8de', text: '0x24272d', muted: '0x66707d', card: '0xf1f3f5', cardOpen: '0xdfe3e8', ring: '0x47679e', selected: '0xdfe3e8', hover: '0xe8ebef', notice: '0xd07277', accent: '0x5c78e2', done: '0x669f59', idle: '0xa48819', quiet: '0x7e8086', terminalForeground: '0x24272d' }, ['0xf9fafb', '0xf4f5f7', '0xf1f3f5', '0xe8ebef']),
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
