import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VideoConferenceIcon` instead.
 *
 * @example
 * ```svelte
 * <VideoConference color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const VideoConference: Component<IconComponentProps, {}, "">;
type VideoConference = ReturnType<typeof VideoConference>;
export default VideoConference;
