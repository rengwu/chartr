import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SubtractIcon` instead.
 *
 * @example
 * ```svelte
 * <Subtract color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Subtract: Component<IconComponentProps, {}, "">;
type Subtract = ReturnType<typeof Subtract>;
export default Subtract;
