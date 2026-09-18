import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextHFiveIcon` instead.
 *
 * @example
 * ```svelte
 * <TextHFive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextHFive: Component<IconComponentProps, {}, "">;
type TextHFive = ReturnType<typeof TextHFive>;
export default TextHFive;
