import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MapPinSimpleAreaIcon` instead.
 *
 * @example
 * ```svelte
 * <MapPinSimpleArea color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MapPinSimpleArea: Component<IconComponentProps, {}, "">;
type MapPinSimpleArea = ReturnType<typeof MapPinSimpleArea>;
export default MapPinSimpleArea;
