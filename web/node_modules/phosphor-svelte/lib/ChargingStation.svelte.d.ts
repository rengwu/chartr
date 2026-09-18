import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChargingStationIcon` instead.
 *
 * @example
 * ```svelte
 * <ChargingStation color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChargingStation: Component<IconComponentProps, {}, "">;
type ChargingStation = ReturnType<typeof ChargingStation>;
export default ChargingStation;
