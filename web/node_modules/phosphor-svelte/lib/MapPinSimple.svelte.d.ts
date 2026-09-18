import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MapPinSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <MapPinSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MapPinSimple: Component<IconComponentProps, {}, "">;
type MapPinSimple = ReturnType<typeof MapPinSimple>;
export default MapPinSimple;
