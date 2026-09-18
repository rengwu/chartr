import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WifiHighIcon` instead.
 *
 * @example
 * ```svelte
 * <WifiHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WifiHigh: Component<IconComponentProps, {}, "">;
type WifiHigh = ReturnType<typeof WifiHigh>;
export default WifiHigh;
