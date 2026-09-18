import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryWarningVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryWarningVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryWarningVertical: Component<IconComponentProps, {}, "">;
type BatteryWarningVertical = ReturnType<typeof BatteryWarningVertical>;
export default BatteryWarningVertical;
