import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowRight: Component<IconComponentProps, {}, "">;
type ArrowRight = ReturnType<typeof ArrowRight>;
export default ArrowRight;
