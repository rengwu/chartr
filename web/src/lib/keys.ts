// Keyboard-first navigation (spec story 30): map summon, space switch, and
// queue summon all have keys, alongside Esc. This one guard is shared by every
// global binding so a keystroke aimed at the terminal's PTY, a text field, or a
// summoned dialog/sheet is never stolen out from under it.
export function isEditingTarget(): boolean {
  const el = document.activeElement as HTMLElement | null
  return (
    !!el &&
    (el.tagName === 'INPUT' ||
      el.tagName === 'TEXTAREA' ||
      el.isContentEditable ||
      el.closest('.terminal-island') !== null ||
      el.closest('[role="dialog"]') !== null)
  )
}
