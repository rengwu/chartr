import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MapPinAreaIcon` instead.
 *
 * @example
 * ```svelte
 * <MapPinArea color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MapPinArea: Component<IconComponentProps, {}, "">;
type MapPinArea = ReturnType<typeof MapPinArea>;
export default MapPinArea;
