// The dot on a tab's card (session-notifications): the quieter half of the same
// event the OS notification carries. A tab that finished a run long enough to be
// worth interrupting the operator over, which they have not looked at since,
// carries `finishedUnseen` in the snapshot — the flag is server state, so the dot
// is there on a reload even though the run may have ended with no browser
// attached at all.
//
// Both derivations below are pure over that one flag plus whether the tab is the
// one in view, and they are exact opposites for a flagged tab: the card shows the
// dot precisely when the operator is not already looking, and the acknowledgement
// posts precisely when they are. That is what keeps a dot from sitting on the tab
// being stared at for the round-trip it takes the server to clear it.
//
// Nothing here touches the attention grammar (`attention.ts`): its `halt` flag and
// its `Liveness` are unchanged, and this signal deliberately never rolls up to a
// space row.

import type { Terminal } from './model'

// Whether this tab's card draws the dot. `inView` is the chrome's own answer to
// "is this the tab currently showing" — the card knows it as the active row of the
// selected space — not something re-derived here.
export function showsFinishedDot(t: Terminal, inView: boolean): boolean {
  return Boolean(t.finishedUnseen) && !inView
}

// Whether looking at this tab acknowledges a finished run — the client's cue to
// post the seen endpoint. Focusing is the only acknowledgement there is: no manual
// dismiss, no clear-all, no count. Null is the no-tab case (a space with no
// terminals, the settings route standing in front of the cockpit), which
// acknowledges nothing.
export function acknowledgesFinishedRun(t: Terminal | null | undefined, inView: boolean): boolean {
  return Boolean(t?.finishedUnseen) && inView
}
