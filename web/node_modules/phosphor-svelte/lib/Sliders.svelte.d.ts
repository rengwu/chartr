import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SlidersIcon` instead.
 *
 * @example
 * ```svelte
 * <Sliders color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Sliders: Component<IconComponentProps, {}, "">;
type Sliders = ReturnType<typeof Sliders>;
export default Sliders;
