import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ColumnsPlusRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ColumnsPlusRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ColumnsPlusRight: Component<IconComponentProps, {}, "">;
type ColumnsPlusRight = ReturnType<typeof ColumnsPlusRight>;
export default ColumnsPlusRight;
