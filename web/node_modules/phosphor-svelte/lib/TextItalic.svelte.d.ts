import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextItalicIcon` instead.
 *
 * @example
 * ```svelte
 * <TextItalic color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextItalic: Component<IconComponentProps, {}, "">;
type TextItalic = ReturnType<typeof TextItalic>;
export default TextItalic;
