import { useMemo, useState } from 'react'
import { Info } from 'lucide-react'
import { HsvColorPicker, type HsvColor } from 'react-colorful'
import { Input } from '@/components/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { normalizeRustHex, toCssHex } from '@/lib/themes'

type Rgb = { r: number; g: number; b: number }

const hexToRgb = (hex: string): Rgb => {
  const value = Number.parseInt(normalizeRustHex(hex).slice(2), 16)
  return { r: (value >> 16) & 255, g: (value >> 8) & 255, b: value & 255 }
}

const rgbToHex = ({ r, g, b }: Rgb) =>
  `0x${[r, g, b].map((channel) => Math.max(0, Math.min(255, Math.round(channel))).toString(16).padStart(2, '0')).join('')}`

const rgbToHsv = ({ r, g, b }: Rgb): HsvColor => {
  const [red, green, blue] = [r, g, b].map((channel) => channel / 255)
  const max = Math.max(red, green, blue)
  const min = Math.min(red, green, blue)
  const delta = max - min
  let h = 0
  if (delta) {
    if (max === red) h = 60 * (((green - blue) / delta) % 6)
    else if (max === green) h = 60 * ((blue - red) / delta + 2)
    else h = 60 * ((red - green) / delta + 4)
  }
  return { h: h < 0 ? h + 360 : h, s: max ? (delta / max) * 100 : 0, v: max * 100 }
}

const hsvToRgb = ({ h, s, v }: HsvColor): Rgb => {
  const saturation = s / 100
  const value = v / 100
  const chroma = value * saturation
  const section = h / 60
  const x = chroma * (1 - Math.abs((section % 2) - 1))
  const match = value - chroma
  const [r, g, b] = section < 1 ? [chroma, x, 0] : section < 2 ? [x, chroma, 0] : section < 3 ? [0, chroma, x] : section < 4 ? [0, x, chroma] : section < 5 ? [x, 0, chroma] : [chroma, 0, x]
  return { r: (r + match) * 255, g: (g + match) * 255, b: (b + match) * 255 }
}

type ColorTokenFieldProps = {
  label: string
  hint: string
  value: string
  onChange: (value: string) => void
}

export function ColorTokenField({ label, hint, value, onChange }: ColorTokenFieldProps) {
  const [draftState, setDraftState] = useState({ source: value, draft: value })
  const draft = draftState.source === value ? draftState.draft : value
  const setDraft = (next: string) => setDraftState({ source: value, draft: next })
  const rgb = useMemo(() => hexToRgb(value), [value])
  const valueHsv = useMemo(() => rgbToHsv(rgb), [rgb])
  const [pickerState, setPickerState] = useState({ source: value, color: valueHsv })
  const hsv = pickerState.source === value ? pickerState.color : valueHsv

  const commitDraft = () => {
    const normalized = normalizeRustHex(draft, value)
    setDraft(normalized)
    onChange(normalized)
  }

  const updateRgb = (channel: keyof Rgb, next: number) => onChange(rgbToHex({ ...rgb, [channel]: next }))

  const updateHsv = (next: HsvColor) => {
    const nextValue = rgbToHex(hsvToRgb(next))

    // Keep react-colorful on its precise pointer coordinates while dragging.
    // Converting the controlled value back through 8-bit RGB on every frame
    // introduces rounding drift, which makes the handle visibly jitter.
    setPickerState({ source: nextValue, color: next })
    onChange(nextValue)
  }

  return (
    <div className="token-row">
      <div className="token-meta">
        <span>{label}</span>
        <Tooltip>
          <TooltipTrigger asChild>
            <button className="token-info" aria-label={`${label}: ${hint}`} type="button"><Info /></button>
          </TooltipTrigger>
          <TooltipContent side="right">{hint}</TooltipContent>
        </Tooltip>
      </div>
      <div className="token-control">
        <Popover>
          <PopoverTrigger asChild>
            <button className="color-swatch" type="button" aria-label={`Edit ${label}`} style={{ background: toCssHex(value) }} />
          </PopoverTrigger>
          <PopoverContent align="start" side="right" className="color-popover">
            <div className="picker-heading">
              <div><strong>{label}</strong><span>{hint}</span></div>
              <code>{value}</code>
            </div>
            <Tabs defaultValue="hsv">
              <TabsList className="picker-tabs">
                <TabsTrigger value="hsv">HSV</TabsTrigger>
                <TabsTrigger value="rgb">RGB</TabsTrigger>
              </TabsList>
              <TabsContent value="hsv" className="picker-panel">
                <HsvColorPicker color={hsv} onChange={updateHsv} />
                <div className="hsv-readout">
                  <span>H <b>{Math.round(hsv.h)}°</b></span>
                  <span>S <b>{Math.round(hsv.s)}%</b></span>
                  <span>V <b>{Math.round(hsv.v)}%</b></span>
                </div>
              </TabsContent>
              <TabsContent value="rgb" className="picker-panel rgb-panel">
                {(['r', 'g', 'b'] as const).map((channel) => (
                  <div className="rgb-row" key={channel}>
                    <label htmlFor={`${label}-${channel}`}>{channel.toUpperCase()}</label>
                    <input
                      id={`${label}-${channel}`}
                      className={`channel-slider channel-${channel}`}
                      type="range"
                      min="0"
                      max="255"
                      value={Math.round(rgb[channel])}
                      onChange={(event) => updateRgb(channel, Number(event.target.value))}
                    />
                    <Input
                      aria-label={`${channel.toUpperCase()} value`}
                      type="number"
                      min="0"
                      max="255"
                      value={Math.round(rgb[channel])}
                      onChange={(event) => updateRgb(channel, Number(event.target.value))}
                    />
                  </div>
                ))}
                <div className="rgb-preview" style={{ background: toCssHex(value) }} />
              </TabsContent>
            </Tabs>
          </PopoverContent>
        </Popover>
        <Input
          className="token-input"
          value={draft}
          aria-label={`${label} Rust color value`}
          spellCheck={false}
          onChange={(event) => {
            const next = event.target.value
            setDraft(next)
            if (/^(0x|#)?[0-9a-f]{6}$/i.test(next)) onChange(normalizeRustHex(next))
          }}
          onBlur={commitDraft}
          onKeyDown={(event) => {
            if (event.key === 'Enter') event.currentTarget.blur()
          }}
        />
      </div>
    </div>
  )
}
