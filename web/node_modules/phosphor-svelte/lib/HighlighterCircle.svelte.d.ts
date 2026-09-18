import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HighlighterCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <HighlighterCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HighlighterCircle: Component<IconComponentProps, {}, "">;
type HighlighterCircle = ReturnType<typeof HighlighterCircle>;
export default HighlighterCircle;
