import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RainbowCloudIcon` instead.
 *
 * @example
 * ```svelte
 * <RainbowCloud color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RainbowCloud: Component<IconComponentProps, {}, "">;
type RainbowCloud = ReturnType<typeof RainbowCloud>;
export default RainbowCloud;
