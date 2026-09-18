import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <SelectionPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionPlus: Component<IconComponentProps, {}, "">;
type SelectionPlus = ReturnType<typeof SelectionPlus>;
export default SelectionPlus;
