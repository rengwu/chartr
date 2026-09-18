import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextStrikethroughIcon` instead.
 *
 * @example
 * ```svelte
 * <TextStrikethrough color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextStrikethrough: Component<IconComponentProps, {}, "">;
type TextStrikethrough = ReturnType<typeof TextStrikethrough>;
export default TextStrikethrough;
