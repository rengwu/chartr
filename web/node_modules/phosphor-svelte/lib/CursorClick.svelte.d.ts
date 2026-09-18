import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CursorClickIcon` instead.
 *
 * @example
 * ```svelte
 * <CursorClick color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CursorClick: Component<IconComponentProps, {}, "">;
type CursorClick = ReturnType<typeof CursorClick>;
export default CursorClick;
