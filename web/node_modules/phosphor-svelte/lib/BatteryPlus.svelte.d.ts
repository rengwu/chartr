import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryPlus: Component<IconComponentProps, {}, "">;
type BatteryPlus = ReturnType<typeof BatteryPlus>;
export default BatteryPlus;
