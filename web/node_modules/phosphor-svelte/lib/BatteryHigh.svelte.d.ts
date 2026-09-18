import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryHighIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryHigh: Component<IconComponentProps, {}, "">;
type BatteryHigh = ReturnType<typeof BatteryHigh>;
export default BatteryHigh;
