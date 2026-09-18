import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <SelectionSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionSlash: Component<IconComponentProps, {}, "">;
type SelectionSlash = ReturnType<typeof SelectionSlash>;
export default SelectionSlash;
