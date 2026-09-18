import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GradientIcon` instead.
 *
 * @example
 * ```svelte
 * <Gradient color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Gradient: Component<IconComponentProps, {}, "">;
type Gradient = ReturnType<typeof Gradient>;
export default Gradient;
