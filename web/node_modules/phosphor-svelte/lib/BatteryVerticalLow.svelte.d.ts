import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryVerticalLowIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryVerticalLow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryVerticalLow: Component<IconComponentProps, {}, "">;
type BatteryVerticalLow = ReturnType<typeof BatteryVerticalLow>;
export default BatteryVerticalLow;
