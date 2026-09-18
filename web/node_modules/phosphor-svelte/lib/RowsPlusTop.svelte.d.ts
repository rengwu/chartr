import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RowsPlusTopIcon` instead.
 *
 * @example
 * ```svelte
 * <RowsPlusTop color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RowsPlusTop: Component<IconComponentProps, {}, "">;
type RowsPlusTop = ReturnType<typeof RowsPlusTop>;
export default RowsPlusTop;
