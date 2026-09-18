import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryWarningIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryWarning color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryWarning: Component<IconComponentProps, {}, "">;
type BatteryWarning = ReturnType<typeof BatteryWarning>;
export default BatteryWarning;
