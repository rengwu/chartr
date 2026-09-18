import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinkSimpleHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <LinkSimpleHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinkSimpleHorizontal: Component<IconComponentProps, {}, "">;
type LinkSimpleHorizontal = ReturnType<typeof LinkSimpleHorizontal>;
export default LinkSimpleHorizontal;
