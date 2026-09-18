import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CursorTextIcon` instead.
 *
 * @example
 * ```svelte
 * <CursorText color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CursorText: Component<IconComponentProps, {}, "">;
type CursorText = ReturnType<typeof CursorText>;
export default CursorText;
