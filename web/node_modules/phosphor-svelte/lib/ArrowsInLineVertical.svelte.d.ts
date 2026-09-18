import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsInLineVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsInLineVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsInLineVertical: Component<IconComponentProps, {}, "">;
type ArrowsInLineVertical = ReturnType<typeof ArrowsInLineVertical>;
export default ArrowsInLineVertical;
