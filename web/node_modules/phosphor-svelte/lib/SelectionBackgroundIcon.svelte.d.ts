import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SelectionBackgroundIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionBackgroundIcon: Component<IconComponentProps, {}, "">;
type SelectionBackgroundIcon = ReturnType<typeof SelectionBackgroundIcon>;
export default SelectionBackgroundIcon;
