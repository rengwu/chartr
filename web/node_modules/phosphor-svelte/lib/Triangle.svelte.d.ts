import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TriangleIcon` instead.
 *
 * @example
 * ```svelte
 * <Triangle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Triangle: Component<IconComponentProps, {}, "">;
type Triangle = ReturnType<typeof Triangle>;
export default Triangle;
