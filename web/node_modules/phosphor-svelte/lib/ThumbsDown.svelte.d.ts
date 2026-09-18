import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThumbsDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ThumbsDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThumbsDown: Component<IconComponentProps, {}, "">;
type ThumbsDown = ReturnType<typeof ThumbsDown>;
export default ThumbsDown;
