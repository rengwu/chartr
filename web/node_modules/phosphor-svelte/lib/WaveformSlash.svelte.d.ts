import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WaveformSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <WaveformSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WaveformSlash: Component<IconComponentProps, {}, "">;
type WaveformSlash = ReturnType<typeof WaveformSlash>;
export default WaveformSlash;
