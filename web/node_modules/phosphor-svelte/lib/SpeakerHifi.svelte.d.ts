import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpeakerHifiIcon` instead.
 *
 * @example
 * ```svelte
 * <SpeakerHifi color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpeakerHifi: Component<IconComponentProps, {}, "">;
type SpeakerHifi = ReturnType<typeof SpeakerHifi>;
export default SpeakerHifi;
