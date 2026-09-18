import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileyWinkIcon` instead.
 *
 * @example
 * ```svelte
 * <SmileyWink color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SmileyWink: Component<IconComponentProps, {}, "">;
type SmileyWink = ReturnType<typeof SmileyWink>;
export default SmileyWink;
