import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TextHTwoIcon` instead.
 *
 * @example
 * ```svelte
 * <TextHTwo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const TextHTwo: Component<IconComponentProps, {}, "">;
type TextHTwo = ReturnType<typeof TextHTwo>;
export default TextHTwo;
