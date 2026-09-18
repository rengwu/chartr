import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextAlignRightIcon` instead.
 *
 * @example
 * ```svelte
 * <TextAlignRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAlignRight: Component<IconComponentProps, {}, "">;
type TextAlignRight = ReturnType<typeof TextAlignRight>;
export default TextAlignRight;
