import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpeakerNoneIcon` instead.
 *
 * @example
 * ```svelte
 * <SpeakerNone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpeakerNone: Component<IconComponentProps, {}, "">;
type SpeakerNone = ReturnType<typeof SpeakerNone>;
export default SpeakerNone;
