import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AirplaneLandingIcon` instead.
 *
 * @example
 * ```svelte
 * <AirplaneLanding color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AirplaneLanding: Component<IconComponentProps, {}, "">;
type AirplaneLanding = ReturnType<typeof AirplaneLanding>;
export default AirplaneLanding;
