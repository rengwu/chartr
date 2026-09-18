import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsVertical: Component<IconComponentProps, {}, "">;
type ArrowsVertical = ReturnType<typeof ArrowsVertical>;
export default ArrowsVertical;
