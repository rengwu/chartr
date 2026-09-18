import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VoicemailIcon` instead.
 *
 * @example
 * ```svelte
 * <Voicemail color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Voicemail: Component<IconComponentProps, {}, "">;
type Voicemail = ReturnType<typeof Voicemail>;
export default Voicemail;
