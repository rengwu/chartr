import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ParagraphIcon` instead.
 *
 * @example
 * ```svelte
 * <Paragraph color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Paragraph: Component<IconComponentProps, {}, "">;
type Paragraph = ReturnType<typeof Paragraph>;
export default Paragraph;
