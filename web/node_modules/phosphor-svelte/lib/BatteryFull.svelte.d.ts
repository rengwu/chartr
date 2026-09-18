import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryFullIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryFull color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryFull: Component<IconComponentProps, {}, "">;
type BatteryFull = ReturnType<typeof BatteryFull>;
export default BatteryFull;
