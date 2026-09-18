import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AlignLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <AlignLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AlignLeft: Component<IconComponentProps, {}, "">;
type AlignLeft = ReturnType<typeof AlignLeft>;
export default AlignLeft;
