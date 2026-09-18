import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WifiNoneIcon` instead.
 *
 * @example
 * ```svelte
 * <WifiNone color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WifiNone: Component<IconComponentProps, {}, "">;
type WifiNone = ReturnType<typeof WifiNone>;
export default WifiNone;
