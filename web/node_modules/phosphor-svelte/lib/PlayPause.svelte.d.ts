import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlayPauseIcon` instead.
 *
 * @example
 * ```svelte
 * <PlayPause color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlayPause: Component<IconComponentProps, {}, "">;
type PlayPause = ReturnType<typeof PlayPause>;
export default PlayPause;
