import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlayCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <PlayCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlayCircle: Component<IconComponentProps, {}, "">;
type PlayCircle = ReturnType<typeof PlayCircle>;
export default PlayCircle;
