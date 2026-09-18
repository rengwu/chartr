import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryLowIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryLow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryLow: Component<IconComponentProps, {}, "">;
type BatteryLow = ReturnType<typeof BatteryLow>;
export default BatteryLow;
