import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThermometerColdIcon` instead.
 *
 * @example
 * ```svelte
 * <ThermometerCold color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThermometerCold: Component<IconComponentProps, {}, "">;
type ThermometerCold = ReturnType<typeof ThermometerCold>;
export default ThermometerCold;
