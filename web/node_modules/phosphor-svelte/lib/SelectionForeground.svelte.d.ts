import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionForegroundIcon` instead.
 *
 * @example
 * ```svelte
 * <SelectionForeground color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionForeground: Component<IconComponentProps, {}, "">;
type SelectionForeground = ReturnType<typeof SelectionForeground>;
export default SelectionForeground;
