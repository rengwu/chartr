import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HighlighterIcon` instead.
 *
 * @example
 * ```svelte
 * <Highlighter color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Highlighter: Component<IconComponentProps, {}, "">;
type Highlighter = ReturnType<typeof Highlighter>;
export default Highlighter;
