import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WifiMediumIcon` instead.
 *
 * @example
 * ```svelte
 * <WifiMedium color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WifiMedium: Component<IconComponentProps, {}, "">;
type WifiMedium = ReturnType<typeof WifiMedium>;
export default WifiMedium;
