import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudRainIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudRain color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudRain: Component<IconComponentProps, {}, "">;
type CloudRain = ReturnType<typeof CloudRain>;
export default CloudRain;
