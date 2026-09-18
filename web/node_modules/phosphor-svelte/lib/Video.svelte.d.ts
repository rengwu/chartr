import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VideoIcon` instead.
 *
 * @example
 * ```svelte
 * <Video color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Video: Component<IconComponentProps, {}, "">;
type Video = ReturnType<typeof Video>;
export default Video;
