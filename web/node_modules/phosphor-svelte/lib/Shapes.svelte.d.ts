import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShapesIcon` instead.
 *
 * @example
 * ```svelte
 * <Shapes color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Shapes: Component<IconComponentProps, {}, "">;
type Shapes = ReturnType<typeof Shapes>;
export default Shapes;
