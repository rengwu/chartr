import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IntersectionIcon` instead.
 *
 * @example
 * ```svelte
 * <Intersection color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Intersection: Component<IconComponentProps, {}, "">;
type Intersection = ReturnType<typeof Intersection>;
export default Intersection;
