import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AmbulanceIcon` instead.
 *
 * @example
 * ```svelte
 * <Ambulance color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ambulance: Component<IconComponentProps, {}, "">;
type Ambulance = ReturnType<typeof Ambulance>;
export default Ambulance;
