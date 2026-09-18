import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinkSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <LinkSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinkSimple: Component<IconComponentProps, {}, "">;
type LinkSimple = ReturnType<typeof LinkSimple>;
export default LinkSimple;
