import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DroneIcon` instead.
 *
 * @example
 * ```svelte
 * <Drone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Drone: Component<IconComponentProps, {}, "">;
type Drone = ReturnType<typeof Drone>;
export default Drone;
