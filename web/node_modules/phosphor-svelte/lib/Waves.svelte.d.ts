import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WavesIcon` instead.
 *
 * @example
 * ```svelte
 * <Waves color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Waves: Component<IconComponentProps, {}, "">;
type Waves = ReturnType<typeof Waves>;
export default Waves;
