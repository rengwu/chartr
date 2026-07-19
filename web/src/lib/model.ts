// The derived model as it arrives over the control socket. This mirrors the Go
// `model.Model` exactly; it is the whole state a snapshot carries, replaced
// wholesale on every push (ADR 0010). Later tickets grow both sides together.

export interface Space {
  id: string
  name: string
}

export interface Model {
  spaces: Space[]
}
