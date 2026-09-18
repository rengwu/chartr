import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextAlignJustifyIcon` instead.
 *
 * @example
 * ```svelte
 * <TextAlignJustify color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAlignJustify: Component<IconComponentProps, {}, "">;
type TextAlignJustify = ReturnType<typeof TextAlignJustify>;
export default TextAlignJustify;
