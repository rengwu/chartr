import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowDownRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowDownRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowDownRight: Component<IconComponentProps, {}, "">;
type ArrowDownRight = ReturnType<typeof ArrowDownRight>;
export default ArrowDownRight;
