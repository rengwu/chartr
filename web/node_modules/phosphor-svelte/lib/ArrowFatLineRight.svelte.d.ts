import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLineRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLineRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLineRight: Component<IconComponentProps, {}, "">;
type ArrowFatLineRight = ReturnType<typeof ArrowFatLineRight>;
export default ArrowFatLineRight;
