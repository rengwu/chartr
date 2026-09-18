import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GridFourIcon` instead.
 *
 * @example
 * ```svelte
 * <GridFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GridFour: Component<IconComponentProps, {}, "">;
type GridFour = ReturnType<typeof GridFour>;
export default GridFour;
