import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AirTrafficControlIcon` instead.
 *
 * @example
 * ```svelte
 * <AirTrafficControl color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AirTrafficControl: Component<IconComponentProps, {}, "">;
type AirTrafficControl = ReturnType<typeof AirTrafficControl>;
export default AirTrafficControl;
