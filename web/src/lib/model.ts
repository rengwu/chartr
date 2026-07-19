// The derived model as it arrives over the control socket. This mirrors the Go
// `model.Model` exactly; it is the whole state a snapshot carries, replaced
// wholesale on every push (ADR 0010). Later tickets grow both sides together.

export type Layer = 'built-in' | 'workspace' | 'user'

// RoleBinding is one role's effective binding: which adapter runs on which
// model with which args, where each field was inherited from (so field-level
// inheritance is visible, story 39), and whether the adapter's binary is on the
// operator's PATH (`missing` carries the absence badge when it is not).
export interface RoleBinding {
  role: string
  adapter: string
  model: string
  args?: string[]
  adapterFrom: Layer
  modelFrom: Layer
  argsFrom: Layer
  present: boolean
  missing?: string
}

export interface Space {
  id: string
  name: string
  path: string
  pinned: boolean
  bindings: RoleBinding[]
  warnings?: string[]
}

export interface Model {
  spaces: Space[]
}

/** A space needs an agent installed if any of its bindings is absent from PATH. */
export function needsAgents(space: Space): boolean {
  return space.bindings.some((b) => !b.present)
}
