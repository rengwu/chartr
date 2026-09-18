import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextAaIcon` instead.
 *
 * @example
 * ```svelte
 * <TextAa color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAa: Component<IconComponentProps, {}, "">;
type TextAa = ReturnType<typeof TextAa>;
export default TextAa;
