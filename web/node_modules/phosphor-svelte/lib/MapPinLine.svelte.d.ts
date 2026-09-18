import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MapPinLineIcon` instead.
 *
 * @example
 * ```svelte
 * <MapPinLine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MapPinLine: Component<IconComponentProps, {}, "">;
type MapPinLine = ReturnType<typeof MapPinLine>;
export default MapPinLine;
