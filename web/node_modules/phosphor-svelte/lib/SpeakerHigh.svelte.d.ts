import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpeakerHighIcon` instead.
 *
 * @example
 * ```svelte
 * <SpeakerHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpeakerHigh: Component<IconComponentProps, {}, "">;
type SpeakerHigh = ReturnType<typeof SpeakerHigh>;
export default SpeakerHigh;
