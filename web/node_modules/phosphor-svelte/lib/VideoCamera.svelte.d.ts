import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VideoCameraIcon` instead.
 *
 * @example
 * ```svelte
 * <VideoCamera color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const VideoCamera: Component<IconComponentProps, {}, "">;
type VideoCamera = ReturnType<typeof VideoCamera>;
export default VideoCamera;
