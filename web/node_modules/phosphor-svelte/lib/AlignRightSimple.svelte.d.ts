import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignRightSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignRightSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignRightSimple: Component<IconComponentProps, {}, "">;
type AlignRightSimple = ReturnType<typeof AlignRightSimple>;
export default AlignRightSimple;
