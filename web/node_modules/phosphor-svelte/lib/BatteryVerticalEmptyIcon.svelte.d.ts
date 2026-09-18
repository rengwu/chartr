import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <BatteryVerticalEmptyIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryVerticalEmptyIcon: Component<IconComponentProps, {}, "">;
type BatteryVerticalEmptyIcon = ReturnType<typeof BatteryVerticalEmptyIcon>;
export default BatteryVerticalEmptyIcon;
