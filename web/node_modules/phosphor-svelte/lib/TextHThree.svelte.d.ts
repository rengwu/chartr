import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextHThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <TextHThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextHThree: Component<IconComponentProps, {}, "">;
type TextHThree = ReturnType<typeof TextHThree>;
export default TextHThree;
