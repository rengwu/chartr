import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextAUnderlineIcon` instead.
 *
 * @example
 * ```svelte
 * <TextAUnderline color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAUnderline: Component<IconComponentProps, {}, "">;
type TextAUnderline = ReturnType<typeof TextAUnderline>;
export default TextAUnderline;
