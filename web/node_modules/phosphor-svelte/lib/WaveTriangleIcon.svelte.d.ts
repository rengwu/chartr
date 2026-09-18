import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <WaveTriangleIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WaveTriangleIcon: Component<IconComponentProps, {}, "">;
type WaveTriangleIcon = ReturnType<typeof WaveTriangleIcon>;
export default WaveTriangleIcon;
