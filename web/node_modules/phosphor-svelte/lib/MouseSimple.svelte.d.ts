import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MouseSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <MouseSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MouseSimple: Component<IconComponentProps, {}, "">;
type MouseSimple = ReturnType<typeof MouseSimple>;
export default MouseSimple;
