import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CrosshairIcon` instead.
 *
 * @example
 * ```svelte
 * <Crosshair color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Crosshair: Component<IconComponentProps, {}, "">;
type Crosshair = ReturnType<typeof Crosshair>;
export default Crosshair;
