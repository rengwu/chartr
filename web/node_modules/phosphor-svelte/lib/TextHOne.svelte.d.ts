import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextHOneIcon` instead.
 *
 * @example
 * ```svelte
 * <TextHOne color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextHOne: Component<IconComponentProps, {}, "">;
type TextHOne = ReturnType<typeof TextHOne>;
export default TextHOne;
