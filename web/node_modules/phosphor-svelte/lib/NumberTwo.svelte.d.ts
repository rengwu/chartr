import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberTwoIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberTwo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberTwo: Component<IconComponentProps, {}, "">;
type NumberTwo = ReturnType<typeof NumberTwo>;
export default NumberTwo;
