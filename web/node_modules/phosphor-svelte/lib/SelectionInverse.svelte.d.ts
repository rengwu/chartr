import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SelectionInverseIcon` instead.
 *
 * @example
 * ```svelte
 * <SelectionInverse color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SelectionInverse: Component<IconComponentProps, {}, "">;
type SelectionInverse = ReturnType<typeof SelectionInverse>;
export default SelectionInverse;
