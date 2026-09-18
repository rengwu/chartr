import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextHFourIcon` instead.
 *
 * @example
 * ```svelte
 * <TextHFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextHFour: Component<IconComponentProps, {}, "">;
type TextHFour = ReturnType<typeof TextHFour>;
export default TextHFour;
