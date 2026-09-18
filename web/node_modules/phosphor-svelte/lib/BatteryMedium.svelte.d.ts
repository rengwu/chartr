import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryMediumIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryMedium color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryMedium: Component<IconComponentProps, {}, "">;
type BatteryMedium = ReturnType<typeof BatteryMedium>;
export default BatteryMedium;
