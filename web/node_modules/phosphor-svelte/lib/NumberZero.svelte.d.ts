import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberZeroIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberZero color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberZero: Component<IconComponentProps, {}, "">;
type NumberZero = ReturnType<typeof NumberZero>;
export default NumberZero;
