import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextAlignLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <TextAlignLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAlignLeft: Component<IconComponentProps, {}, "">;
type TextAlignLeft = ReturnType<typeof TextAlignLeft>;
export default TextAlignLeft;
