import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotchesIcon` instead.
 *
 * @example
 * ```svelte
 * <Notches color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Notches: Component<IconComponentProps, {}, "">;
type Notches = ReturnType<typeof Notches>;
export default Notches;
