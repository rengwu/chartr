import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberFourIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberFour: Component<IconComponentProps, {}, "">;
type NumberFour = ReturnType<typeof NumberFour>;
export default NumberFour;
