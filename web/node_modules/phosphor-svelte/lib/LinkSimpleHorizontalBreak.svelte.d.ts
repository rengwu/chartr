import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinkSimpleHorizontalBreakIcon` instead.
 *
 * @example
 * ```svelte
 * <LinkSimpleHorizontalBreak color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinkSimpleHorizontalBreak: Component<IconComponentProps, {}, "">;
type LinkSimpleHorizontalBreak = ReturnType<typeof LinkSimpleHorizontalBreak>;
export default LinkSimpleHorizontalBreak;
