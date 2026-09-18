import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ParachuteIcon` instead.
 *
 * @example
 * ```svelte
 * <Parachute color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Parachute: Component<IconComponentProps, {}, "">;
type Parachute = ReturnType<typeof Parachute>;
export default Parachute;
