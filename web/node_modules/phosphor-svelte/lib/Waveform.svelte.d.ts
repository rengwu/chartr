import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WaveformIcon` instead.
 *
 * @example
 * ```svelte
 * <Waveform color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Waveform: Component<IconComponentProps, {}, "">;
type Waveform = ReturnType<typeof Waveform>;
export default Waveform;
