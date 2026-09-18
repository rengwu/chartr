import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberFiveIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberFive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberFive: Component<IconComponentProps, {}, "">;
type NumberFive = ReturnType<typeof NumberFive>;
export default NumberFive;
