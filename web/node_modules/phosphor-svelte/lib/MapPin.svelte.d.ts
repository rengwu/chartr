import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MapPinIcon` instead.
 *
 * @example
 * ```svelte
 * <MapPin color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MapPin: Component<IconComponentProps, {}, "">;
type MapPin = ReturnType<typeof MapPin>;
export default MapPin;
