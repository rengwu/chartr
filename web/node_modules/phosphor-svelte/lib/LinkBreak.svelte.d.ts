import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinkBreakIcon` instead.
 *
 * @example
 * ```svelte
 * <LinkBreak color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinkBreak: Component<IconComponentProps, {}, "">;
type LinkBreak = ReturnType<typeof LinkBreak>;
export default LinkBreak;
