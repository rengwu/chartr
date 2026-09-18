import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpeakerSimpleHighIcon` instead.
 *
 * @example
 * ```svelte
 * <SpeakerSimpleHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpeakerSimpleHigh: Component<IconComponentProps, {}, "">;
type SpeakerSimpleHigh = ReturnType<typeof SpeakerSimpleHigh>;
export default SpeakerSimpleHigh;
