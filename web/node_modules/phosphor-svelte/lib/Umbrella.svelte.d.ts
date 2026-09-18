import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `UmbrellaIcon` instead.
 *
 * @example
 * ```svelte
 * <Umbrella color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Umbrella: Component<IconComponentProps, {}, "">;
type Umbrella = ReturnType<typeof Umbrella>;
export default Umbrella;
