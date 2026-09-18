import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryVerticalEmptyIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryVerticalEmpty color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryVerticalEmpty: Component<IconComponentProps, {}, "">;
type BatteryVerticalEmpty = ReturnType<typeof BatteryVerticalEmpty>;
export default BatteryVerticalEmpty;
