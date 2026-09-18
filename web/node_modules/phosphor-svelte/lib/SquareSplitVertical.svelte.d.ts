import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SquareSplitVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <SquareSplitVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SquareSplitVertical: Component<IconComponentProps, {}, "">;
type SquareSplitVertical = ReturnType<typeof SquareSplitVertical>;
export default SquareSplitVertical;
