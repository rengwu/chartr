import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandHeartIcon` instead.
 *
 * @example
 * ```svelte
 * <HandHeart color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandHeart: Component<IconComponentProps, {}, "">;
type HandHeart = ReturnType<typeof HandHeart>;
export default HandHeart;
