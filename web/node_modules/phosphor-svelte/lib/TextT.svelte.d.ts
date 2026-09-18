import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextTIcon` instead.
 *
 * @example
 * ```svelte
 * <TextT color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextT: Component<IconComponentProps, {}, "">;
type TextT = ReturnType<typeof TextT>;
export default TextT;
