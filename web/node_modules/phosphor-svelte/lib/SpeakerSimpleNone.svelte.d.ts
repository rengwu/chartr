import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpeakerSimpleNoneIcon` instead.
 *
 * @example
 * ```svelte
 * <SpeakerSimpleNone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SpeakerSimpleNone: Component<IconComponentProps, {}, "">;
type SpeakerSimpleNone = ReturnType<typeof SpeakerSimpleNone>;
export default SpeakerSimpleNone;
