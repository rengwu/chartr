import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RowsPlusBottomIcon` instead.
 *
 * @example
 * ```svelte
 * <RowsPlusBottom color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RowsPlusBottom: Component<IconComponentProps, {}, "">;
type RowsPlusBottom = ReturnType<typeof RowsPlusBottom>;
export default RowsPlusBottom;
