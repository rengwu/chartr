import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BuildingApartmentIcon` instead.
 *
 * @example
 * ```svelte
 * <BuildingApartment color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BuildingApartment: Component<IconComponentProps, {}, "">;
type BuildingApartment = ReturnType<typeof BuildingApartment>;
export default BuildingApartment;
