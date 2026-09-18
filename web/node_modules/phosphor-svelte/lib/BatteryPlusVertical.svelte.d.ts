import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryPlusVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryPlusVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryPlusVertical: Component<IconComponentProps, {}, "">;
type BatteryPlusVertical = ReturnType<typeof BatteryPlusVertical>;
export default BatteryPlusVertical;
