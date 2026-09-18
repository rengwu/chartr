import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicrophoneIcon` instead.
 *
 * @example
 * ```svelte
 * <Microphone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Microphone: Component<IconComponentProps, {}, "">;
type Microphone = ReturnType<typeof Microphone>;
export default Microphone;
