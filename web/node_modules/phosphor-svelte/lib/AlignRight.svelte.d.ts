import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignRightIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignRight: Component<IconComponentProps, {}, "">;
type AlignRight = ReturnType<typeof AlignRight>;
export default AlignRight;
