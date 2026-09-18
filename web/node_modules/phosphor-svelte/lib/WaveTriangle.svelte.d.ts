import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WaveTriangleIcon` instead.
 *
 * @example
 * ```svelte
 * <WaveTriangle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WaveTriangle: Component<IconComponentProps, {}, "">;
type WaveTriangle = ReturnType<typeof WaveTriangle>;
export default WaveTriangle;
