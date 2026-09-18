import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PulseIcon` instead.
 *
 * @example
 * ```svelte
 * <Pulse color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pulse: Component<IconComponentProps, {}, "">;
type Pulse = ReturnType<typeof Pulse>;
export default Pulse;
