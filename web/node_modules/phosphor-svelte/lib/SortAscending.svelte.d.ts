import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SortAscendingIcon` instead.
 *
 * @example
 * ```svelte
 * <SortAscending color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SortAscending: Component<IconComponentProps, {}, "">;
type SortAscending = ReturnType<typeof SortAscending>;
export default SortAscending;
