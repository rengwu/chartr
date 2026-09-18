import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MetronomeIcon` instead.
 *
 * @example
 * ```svelte
 * <Metronome color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Metronome: Component<IconComponentProps, {}, "">;
type Metronome = ReturnType<typeof Metronome>;
export default Metronome;
