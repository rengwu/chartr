import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AirplaneTakeoffIcon` instead.
 *
 * @example
 * ```svelte
 * <AirplaneTakeoff color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AirplaneTakeoff: Component<IconComponentProps, {}, "">;
type AirplaneTakeoff = ReturnType<typeof AirplaneTakeoff>;
export default AirplaneTakeoff;
