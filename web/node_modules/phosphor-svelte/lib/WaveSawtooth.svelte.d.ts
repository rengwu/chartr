import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WaveSawtoothIcon` instead.
 *
 * @example
 * ```svelte
 * <WaveSawtooth color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WaveSawtooth: Component<IconComponentProps, {}, "">;
type WaveSawtooth = ReturnType<typeof WaveSawtooth>;
export default WaveSawtooth;
