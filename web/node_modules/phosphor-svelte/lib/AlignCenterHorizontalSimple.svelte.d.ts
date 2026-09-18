import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignCenterHorizontalSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignCenterHorizontalSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignCenterHorizontalSimple: Component<IconComponentProps, {}, "">;
type AlignCenterHorizontalSimple = ReturnType<typeof AlignCenterHorizontalSimple>;
export default AlignCenterHorizontalSimple;
