import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MapPinPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <MapPinPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MapPinPlus: Component<IconComponentProps, {}, "">;
type MapPinPlus = ReturnType<typeof MapPinPlus>;
export default MapPinPlus;
