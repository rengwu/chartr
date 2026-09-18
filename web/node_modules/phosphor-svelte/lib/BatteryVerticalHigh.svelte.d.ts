import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryVerticalHighIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryVerticalHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryVerticalHigh: Component<IconComponentProps, {}, "">;
type BatteryVerticalHigh = ReturnType<typeof BatteryVerticalHigh>;
export default BatteryVerticalHigh;
