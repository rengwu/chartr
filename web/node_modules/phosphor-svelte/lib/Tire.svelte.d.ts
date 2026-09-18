import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TireIcon` instead.
 *
 * @example
 * ```svelte
 * <Tire color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Tire: Component<IconComponentProps, {}, "">;
type Tire = ReturnType<typeof Tire>;
export default Tire;
