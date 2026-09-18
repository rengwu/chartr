import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SubsetOfIcon` instead.
 *
 * @example
 * ```svelte
 * <SubsetOf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SubsetOf: Component<IconComponentProps, {}, "">;
type SubsetOf = ReturnType<typeof SubsetOf>;
export default SubsetOf;
