import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignLeftSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignLeftSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignLeftSimple: Component<IconComponentProps, {}, "">;
type AlignLeftSimple = ReturnType<typeof AlignLeftSimple>;
export default AlignLeftSimple;
