import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryEmptyIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryEmpty color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryEmpty: Component<IconComponentProps, {}, "">;
type BatteryEmpty = ReturnType<typeof BatteryEmpty>;
export default BatteryEmpty;
