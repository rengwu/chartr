import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WaveSineIcon` instead.
 *
 * @example
 * ```svelte
 * <WaveSine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WaveSine: Component<IconComponentProps, {}, "">;
type WaveSine = ReturnType<typeof WaveSine>;
export default WaveSine;
