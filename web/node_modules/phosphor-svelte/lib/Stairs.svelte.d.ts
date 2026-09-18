import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StairsIcon` instead.
 *
 * @example
 * ```svelte
 * <Stairs color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Stairs: Component<IconComponentProps, {}, "">;
type Stairs = ReturnType<typeof Stairs>;
export default Stairs;
