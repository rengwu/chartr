import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignCenterHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignCenterHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignCenterHorizontal: Component<IconComponentProps, {}, "">;
type AlignCenterHorizontal = ReturnType<typeof AlignCenterHorizontal>;
export default AlignCenterHorizontal;
