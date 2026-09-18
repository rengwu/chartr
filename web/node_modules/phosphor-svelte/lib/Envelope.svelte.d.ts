import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EnvelopeIcon` instead.
 *
 * @example
 * ```svelte
 * <Envelope color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Envelope: Component<IconComponentProps, {}, "">;
type Envelope = ReturnType<typeof Envelope>;
export default Envelope;
