import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLinesRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLinesRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLinesRight: Component<IconComponentProps, {}, "">;
type ArrowFatLinesRight = ReturnType<typeof ArrowFatLinesRight>;
export default ArrowFatLinesRight;
