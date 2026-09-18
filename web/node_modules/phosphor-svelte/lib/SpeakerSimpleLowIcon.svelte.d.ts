import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SpeakerSimpleLowIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpeakerSimpleLowIcon: Component<IconComponentProps, {}, "">;
type SpeakerSimpleLowIcon = ReturnType<typeof SpeakerSimpleLowIcon>;
export default SpeakerSimpleLowIcon;
