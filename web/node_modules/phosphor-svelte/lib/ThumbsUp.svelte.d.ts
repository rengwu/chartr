import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThumbsUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ThumbsUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThumbsUp: Component<IconComponentProps, {}, "">;
type ThumbsUp = ReturnType<typeof ThumbsUp>;
export default ThumbsUp;
