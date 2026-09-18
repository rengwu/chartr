import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GridNineIcon` instead.
 *
 * @example
 * ```svelte
 * <GridNine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GridNine: Component<IconComponentProps, {}, "">;
type GridNine = ReturnType<typeof GridNine>;
export default GridNine;
