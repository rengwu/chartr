import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ScooterIcon` instead.
 *
 * @example
 * ```svelte
 * <Scooter color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Scooter: Component<IconComponentProps, {}, "">;
type Scooter = ReturnType<typeof Scooter>;
export default Scooter;
