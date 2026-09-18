import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RectangleIcon` instead.
 *
 * @example
 * ```svelte
 * <Rectangle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Rectangle: Component<IconComponentProps, {}, "">;
type Rectangle = ReturnType<typeof Rectangle>;
export default Rectangle;
