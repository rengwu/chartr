import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowUUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowUUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowUUpRight: Component<IconComponentProps, {}, "">;
type ArrowUUpRight = ReturnType<typeof ArrowUUpRight>;
export default ArrowUUpRight;
