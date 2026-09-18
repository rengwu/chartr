import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignBottomSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignBottomSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignBottomSimple: Component<IconComponentProps, {}, "">;
type AlignBottomSimple = ReturnType<typeof AlignBottomSimple>;
export default AlignBottomSimple;
