import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SkipForwardIcon` instead.
 *
 * @example
 * ```svelte
 * <SkipForward color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SkipForward: Component<IconComponentProps, {}, "">;
type SkipForward = ReturnType<typeof SkipForward>;
export default SkipForward;
