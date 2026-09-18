import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextSubscriptIcon` instead.
 *
 * @example
 * ```svelte
 * <TextSubscript color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextSubscript: Component<IconComponentProps, {}, "">;
type TextSubscript = ReturnType<typeof TextSubscript>;
export default TextSubscript;
