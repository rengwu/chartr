import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextHIcon` instead.
 *
 * @example
 * ```svelte
 * <TextH color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextH: Component<IconComponentProps, {}, "">;
type TextH = ReturnType<typeof TextH>;
export default TextH;
