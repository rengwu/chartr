import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionBackgroundIcon` instead.
 *
 * @example
 * ```svelte
 * <SelectionBackground color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionBackground: Component<IconComponentProps, {}, "">;
type SelectionBackground = ReturnType<typeof SelectionBackground>;
export default SelectionBackground;
