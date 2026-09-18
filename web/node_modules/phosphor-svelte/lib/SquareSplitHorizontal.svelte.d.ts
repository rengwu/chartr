import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SquareSplitHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <SquareSplitHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SquareSplitHorizontal: Component<IconComponentProps, {}, "">;
type SquareSplitHorizontal = ReturnType<typeof SquareSplitHorizontal>;
export default SquareSplitHorizontal;
