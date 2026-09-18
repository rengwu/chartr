import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandSwipeLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <HandSwipeLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandSwipeLeft: Component<IconComponentProps, {}, "">;
type HandSwipeLeft = ReturnType<typeof HandSwipeLeft>;
export default HandSwipeLeft;
