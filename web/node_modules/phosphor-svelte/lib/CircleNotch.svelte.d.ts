import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CircleNotchIcon` instead.
 *
 * @example
 * ```svelte
 * <CircleNotch color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CircleNotch: Component<IconComponentProps, {}, "">;
type CircleNotch = ReturnType<typeof CircleNotch>;
export default CircleNotch;
