import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MouseRightClickIcon` instead.
 *
 * @example
 * ```svelte
 * <MouseRightClick color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MouseRightClick: Component<IconComponentProps, {}, "">;
type MouseRightClick = ReturnType<typeof MouseRightClick>;
export default MouseRightClick;
