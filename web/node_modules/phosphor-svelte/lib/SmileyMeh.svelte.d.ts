import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileyMehIcon` instead.
 *
 * @example
 * ```svelte
 * <SmileyMeh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SmileyMeh: Component<IconComponentProps, {}, "">;
type SmileyMeh = ReturnType<typeof SmileyMeh>;
export default SmileyMeh;
