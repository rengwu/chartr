import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EyeglassesIcon` instead.
 *
 * @example
 * ```svelte
 * <Eyeglasses color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Eyeglasses: Component<IconComponentProps, {}, "">;
type Eyeglasses = ReturnType<typeof Eyeglasses>;
export default Eyeglasses;
