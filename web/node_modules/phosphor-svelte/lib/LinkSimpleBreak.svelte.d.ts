import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinkSimpleBreakIcon` instead.
 *
 * @example
 * ```svelte
 * <LinkSimpleBreak color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinkSimpleBreak: Component<IconComponentProps, {}, "">;
type LinkSimpleBreak = ReturnType<typeof LinkSimpleBreak>;
export default LinkSimpleBreak;
