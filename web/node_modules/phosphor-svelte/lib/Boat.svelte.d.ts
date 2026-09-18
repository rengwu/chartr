import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BoatIcon` instead.
 *
 * @example
 * ```svelte
 * <Boat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Boat: Component<IconComponentProps, {}, "">;
type Boat = ReturnType<typeof Boat>;
export default Boat;
