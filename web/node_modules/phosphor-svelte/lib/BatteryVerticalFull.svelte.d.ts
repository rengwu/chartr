import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryVerticalFullIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryVerticalFull color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryVerticalFull: Component<IconComponentProps, {}, "">;
type BatteryVerticalFull = ReturnType<typeof BatteryVerticalFull>;
export default BatteryVerticalFull;
