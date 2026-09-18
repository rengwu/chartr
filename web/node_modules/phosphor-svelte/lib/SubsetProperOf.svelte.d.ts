import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SubsetProperOfIcon` instead.
 *
 * @example
 * ```svelte
 * <SubsetProperOf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SubsetProperOf: Component<IconComponentProps, {}, "">;
type SubsetProperOf = ReturnType<typeof SubsetProperOf>;
export default SubsetProperOf;
