import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsOutLineVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsOutLineVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsOutLineVertical: Component<IconComponentProps, {}, "">;
type ArrowsOutLineVertical = ReturnType<typeof ArrowsOutLineVertical>;
export default ArrowsOutLineVertical;
