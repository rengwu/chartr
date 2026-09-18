import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ColumnsPlusLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ColumnsPlusLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ColumnsPlusLeft: Component<IconComponentProps, {}, "">;
type ColumnsPlusLeft = ReturnType<typeof ColumnsPlusLeft>;
export default ColumnsPlusLeft;
