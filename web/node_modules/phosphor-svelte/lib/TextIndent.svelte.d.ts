import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextIndentIcon` instead.
 *
 * @example
 * ```svelte
 * <TextIndent color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextIndent: Component<IconComponentProps, {}, "">;
type TextIndent = ReturnType<typeof TextIndent>;
export default TextIndent;
