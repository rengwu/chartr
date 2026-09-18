import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FadersIcon` instead.
 *
 * @example
 * ```svelte
 * <Faders color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Faders: Component<IconComponentProps, {}, "">;
type Faders = ReturnType<typeof Faders>;
export default Faders;
