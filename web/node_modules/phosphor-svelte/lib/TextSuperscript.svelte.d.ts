import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextSuperscriptIcon` instead.
 *
 * @example
 * ```svelte
 * <TextSuperscript color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextSuperscript: Component<IconComponentProps, {}, "">;
type TextSuperscript = ReturnType<typeof TextSuperscript>;
export default TextSuperscript;
