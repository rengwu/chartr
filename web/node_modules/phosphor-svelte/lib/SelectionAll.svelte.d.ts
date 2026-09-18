import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionAllIcon` instead.
 *
 * @example
 * ```svelte
 * <SelectionAll color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionAll: Component<IconComponentProps, {}, "">;
type SelectionAll = ReturnType<typeof SelectionAll>;
export default SelectionAll;
