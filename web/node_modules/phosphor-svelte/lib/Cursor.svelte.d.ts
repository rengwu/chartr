import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CursorIcon` instead.
 *
 * @example
 * ```svelte
 * <Cursor color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cursor: Component<IconComponentProps, {}, "">;
type Cursor = ReturnType<typeof Cursor>;
export default Cursor;
