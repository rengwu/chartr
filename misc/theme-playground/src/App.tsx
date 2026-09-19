import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import { Braces, Check, Copy, Download, FileDown, Import, RotateCcw, Sparkles } from 'lucide-react'
import { ColorTokenField } from '@/components/color-token-field'
import { ThemePreview } from '@/components/chartr-preview'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { TooltipProvider } from '@/components/ui/tooltip'
import {
  clonePreset,
  makeRustExport,
  THEME_PRESETS,
  TOKEN_GROUPS,
  toCssHex,
  type Appearance,
  type ThemePreset,
  type ThemeTokens,
  type TokenKey,
} from '@/lib/themes'
import './App.css'

const STORAGE_KEY = 'chartr-theme-playground:draft-v1'
const DEFAULT_THEME = THEME_PRESETS.find((theme) => theme.name === 'chartr Dark') ?? THEME_PRESETS[0]

const loadTheme = (): ThemePreset => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) return JSON.parse(saved) as ThemePreset
  } catch {
    // Ignore a stale or malformed local draft.
  }
  return clonePreset(DEFAULT_THEME)
}

const downloadText = (filename: string, content: string, type: string) => {
  const blob = new Blob([content], { type })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

const fileSlug = (name: string) => name.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'chartr-theme'

const PREVIEW_VARIABLES: Record<keyof ThemeTokens, string> = {
  surface: '--t-surface', sidebar: '--t-sidebar', border: '--t-border', text: '--t-text', muted: '--t-muted', card: '--t-card', cardOpen: '--t-card-open', ring: '--t-ring', selected: '--t-selected', hover: '--t-hover', notice: '--t-notice', accent: '--t-accent', done: '--t-done', idle: '--t-idle', quiet: '--t-quiet', terminalForeground: '--t-terminal', sidebarCardInactive: '--t-side-card-inactive', sidebarCardActive: '--t-side-card-active', sidebarSessionHover: '--t-side-session-hover', sidebarSessionActive: '--t-side-session-active',
}

function App() {
  const initialTheme = useMemo(() => loadTheme(), [])
  const [theme, setTheme] = useState<ThemePreset>(initialTheme)
  const [sourcePreset, setSourcePreset] = useState(() => THEME_PRESETS.some((preset) => preset.name === initialTheme.name) ? initialTheme.name : 'custom')
  const [copied, setCopied] = useState(false)
  const fileInput = useRef<HTMLInputElement>(null)

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(theme))
  }, [theme])

  const currentPreset = THEME_PRESETS.find((preset) => preset.name === sourcePreset)
  const isModified = !currentPreset || JSON.stringify(theme) !== JSON.stringify(currentPreset)
  const rustExport = useMemo(() => makeRustExport(theme), [theme])
  const previewStyle = useMemo(() => {
    const values = Object.entries(theme.tokens).map(([key, value]) => [PREVIEW_VARIABLES[key as TokenKey], toCssHex(value)])
    return Object.fromEntries(values) as CSSProperties
  }, [theme.tokens])

  const choosePreset = (name: string) => {
    const preset = THEME_PRESETS.find((candidate) => candidate.name === name)
    if (!preset) return
    setTheme(clonePreset(preset))
    setSourcePreset(name)
  }

  const updateToken = (key: TokenKey, value: string) => {
    setTheme((current) => ({ ...current, tokens: { ...current.tokens, [key]: value } }))
  }

  const importTheme = async (file: File) => {
    try {
      const imported = JSON.parse(await file.text()) as Partial<ThemePreset>
      const hasTokens = imported.tokens && Object.keys(DEFAULT_THEME.tokens).every((key) => typeof imported.tokens?.[key as TokenKey] === 'string')
      if (!hasTokens) return
      setTheme({
        name: typeof imported.name === 'string' ? imported.name : 'Imported theme',
        appearance: imported.appearance === 'Light' ? 'Light' : 'Dark',
        tokens: imported.tokens as ThemeTokens,
      })
      setSourcePreset('custom')
    } finally {
      if (fileInput.current) fileInput.current.value = ''
    }
  }

  const copyRust = async () => {
    await navigator.clipboard.writeText(rustExport)
    setCopied(true)
    window.setTimeout(() => setCopied(false), 1600)
  }

  return (
    <TooltipProvider delayDuration={250}>
      <div className="playground-shell">
        <aside className="builder-sidebar">
          <header className="builder-brand">
            <div className="brand-mark"><Braces /></div>
            <div><strong>Theme Playground</strong><span>chartr developer tool</span></div>
            <span className="beta-badge">DEV</span>
          </header>

          <div className="builder-scroll">
            <section className="preset-section">
              <div className="section-kicker"><span>STARTING POINT</span>{isModified && <i>Modified</i>}</div>
              <Select value={sourcePreset === 'custom' ? undefined : sourcePreset} onValueChange={choosePreset}>
                <SelectTrigger className="preset-trigger" aria-label="Theme preset"><SelectValue placeholder="Custom / imported theme" /></SelectTrigger>
                <SelectContent>
                  {THEME_PRESETS.map((preset) => (
                    <SelectItem value={preset.name} key={preset.name}>
                      <span className="preset-option"><i style={{ background: toCssHex(preset.tokens.surface), borderColor: toCssHex(preset.tokens.border) }}><b style={{ background: toCssHex(preset.tokens.accent) }} /></i>{preset.name}<em>{preset.appearance}</em></span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <div className="identity-grid">
                <label><span>Theme name</span><Input value={theme.name} onChange={(event) => setTheme((current) => ({ ...current, name: event.target.value }))} /></label>
                <label><span>Appearance</span><div className="appearance-toggle">{(['Dark', 'Light'] as Appearance[]).map((appearance) => <button type="button" className={theme.appearance === appearance ? 'active' : ''} key={appearance} onClick={() => setTheme((current) => ({ ...current, appearance }))}>{appearance}</button>)}</div></label>
              </div>
            </section>

            <div className="token-intro"><div><span>COLOR TOKENS</span><b>{Object.keys(theme.tokens).length}</b></div><p>Click a swatch for HSV and RGB controls. Values map directly to Rust.</p></div>

            {TOKEN_GROUPS.map((group) => (
              <section className="token-group" key={group.label}>
                <div className="token-group-title"><div><strong>{group.label}</strong><span>{group.description}</span></div><b>{group.tokens.length}</b></div>
                <div className="token-list">
                  {group.tokens.map(({ key, label, hint }) => <ColorTokenField key={key} label={label} hint={hint} value={theme.tokens[key]} onChange={(value) => updateToken(key, value)} />)}
                </div>
              </section>
            ))}
          </div>

          <footer className="builder-actions">
            <input ref={fileInput} className="sr-only" type="file" accept="application/json,.json" onChange={(event) => event.target.files?.[0] && void importTheme(event.target.files[0])} />
            <Button variant="outline" size="icon" aria-label="Import theme JSON" onClick={() => fileInput.current?.click()}><Import /></Button>
            <Button variant="outline" size="icon" aria-label="Reset to preset" disabled={!currentPreset} onClick={() => currentPreset && setTheme(clonePreset(currentPreset))}><RotateCcw /></Button>
            <Button variant="outline" size="icon" aria-label="Download theme JSON" onClick={() => downloadText(`${fileSlug(theme.name)}.json`, JSON.stringify(theme, null, 2), 'application/json')}><Download /></Button>
            <Dialog>
              <DialogTrigger asChild><Button className="export-button"><FileDown /> Export to Rust</Button></DialogTrigger>
              <DialogContent>
                <div className="export-heading"><div className="export-icon"><Sparkles /></div><div><DialogTitle>Register {theme.name || 'new theme'}</DialogTitle><DialogDescription>Paste both entries into <code>crates/chartr/src/settings.rs</code>, bump the two fixed array lengths, and the theme will be registered on the next build.</DialogDescription></div></div>
                <pre className="rust-output"><code>{rustExport}</code></pre>
                <div className="export-actions">
                  <Button variant="outline" onClick={() => downloadText(`${fileSlug(theme.name)}.rs`, rustExport, 'text/plain')}><FileDown /> Download .rs</Button>
                  <Button onClick={() => void copyRust()}>{copied ? <Check /> : <Copy />}{copied ? 'Copied' : 'Copy Rust entries'}</Button>
                </div>
              </DialogContent>
            </Dialog>
          </footer>
        </aside>

        <main className="preview-area" style={previewStyle}>
          <header className="playground-topbar">
            <div><span>PREVIEWING</span><strong>{theme.name || 'Untitled theme'}</strong><i>{theme.appearance}</i></div>
            <div className="palette-strip" aria-label="Theme palette">{['surface', 'sidebar', 'card', 'border', 'muted', 'text', 'accent', 'done', 'idle', 'notice'].map((key) => <i key={key} title={key} style={{ background: toCssHex(theme.tokens[key as TokenKey]) }} />)}</div>
          </header>
          <ThemePreview />
        </main>
      </div>
    </TooltipProvider>
  )
}

export default App
