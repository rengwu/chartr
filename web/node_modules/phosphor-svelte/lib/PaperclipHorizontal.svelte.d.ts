import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaperclipHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <PaperclipHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PaperclipHorizontal: Component<IconComponentProps, {}, "">;
type PaperclipHorizontal = ReturnType<typeof PaperclipHorizontal>;
export default PaperclipHorizontal;
