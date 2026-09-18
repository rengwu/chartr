import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ThumbsUpIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThumbsUpIcon: Component<IconComponentProps, {}, "">;
type ThumbsUpIcon = ReturnType<typeof ThumbsUpIcon>;
export default ThumbsUpIcon;
