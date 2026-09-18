import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <TextAlignLeftIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAlignLeftIcon: Component<IconComponentProps, {}, "">;
type TextAlignLeftIcon = ReturnType<typeof TextAlignLeftIcon>;
export default TextAlignLeftIcon;
