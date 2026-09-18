import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <VideoConferenceIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const VideoConferenceIcon: Component<IconComponentProps, {}, "">;
type VideoConferenceIcon = ReturnType<typeof VideoConferenceIcon>;
export default VideoConferenceIcon;
