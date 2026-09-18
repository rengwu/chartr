import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextAlignCenterIcon` instead.
 *
 * @example
 * ```svelte
 * <TextAlignCenter color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextAlignCenter: Component<IconComponentProps, {}, "">;
type TextAlignCenter = ReturnType<typeof TextAlignCenter>;
export default TextAlignCenter;
