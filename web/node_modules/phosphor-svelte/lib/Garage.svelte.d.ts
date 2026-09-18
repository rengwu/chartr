import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GarageIcon` instead.
 *
 * @example
 * ```svelte
 * <Garage color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Garage: Component<IconComponentProps, {}, "">;
type Garage = ReturnType<typeof Garage>;
export default Garage;
