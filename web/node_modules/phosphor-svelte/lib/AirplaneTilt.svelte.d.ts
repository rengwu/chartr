import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AirplaneTiltIcon` instead.
 *
 * @example
 * ```svelte
 * <AirplaneTilt color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AirplaneTilt: Component<IconComponentProps, {}, "">;
type AirplaneTilt = ReturnType<typeof AirplaneTilt>;
export default AirplaneTilt;
