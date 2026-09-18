import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <AirplaneTakeoffIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AirplaneTakeoffIcon: Component<IconComponentProps, {}, "">;
type AirplaneTakeoffIcon = ReturnType<typeof AirplaneTakeoffIcon>;
export default AirplaneTakeoffIcon;
