import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSevenIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSeven color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSeven: Component<IconComponentProps, {}, "">;
type NumberSeven = ReturnType<typeof NumberSeven>;
export default NumberSeven;
