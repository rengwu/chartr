import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MouseMiddleClickIcon` instead.
 *
 * @example
 * ```svelte
 * <MouseMiddleClick color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MouseMiddleClick: Component<IconComponentProps, {}, "">;
type MouseMiddleClick = ReturnType<typeof MouseMiddleClick>;
export default MouseMiddleClick;
