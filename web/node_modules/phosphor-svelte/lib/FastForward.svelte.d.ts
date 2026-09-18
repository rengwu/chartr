import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FastForwardIcon` instead.
 *
 * @example
 * ```svelte
 * <FastForward color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FastForward: Component<IconComponentProps, {}, "">;
type FastForward = ReturnType<typeof FastForward>;
export default FastForward;
