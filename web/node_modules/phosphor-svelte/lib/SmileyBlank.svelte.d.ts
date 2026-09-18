import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileyBlankIcon` instead.
 *
 * @example
 * ```svelte
 * <SmileyBlank color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SmileyBlank: Component<IconComponentProps, {}, "">;
type SmileyBlank = ReturnType<typeof SmileyBlank>;
export default SmileyBlank;
