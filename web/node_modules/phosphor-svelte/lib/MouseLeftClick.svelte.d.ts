import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MouseLeftClickIcon` instead.
 *
 * @example
 * ```svelte
 * <MouseLeftClick color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MouseLeftClick: Component<IconComponentProps, {}, "">;
type MouseLeftClick = ReturnType<typeof MouseLeftClick>;
export default MouseLeftClick;
