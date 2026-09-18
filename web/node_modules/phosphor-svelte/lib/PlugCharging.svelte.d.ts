import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlugChargingIcon` instead.
 *
 * @example
 * ```svelte
 * <PlugCharging color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlugCharging: Component<IconComponentProps, {}, "">;
type PlugCharging = ReturnType<typeof PlugCharging>;
export default PlugCharging;
