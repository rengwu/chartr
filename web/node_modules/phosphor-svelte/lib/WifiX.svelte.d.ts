import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WifiXIcon` instead.
 *
 * @example
 * ```svelte
 * <WifiX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WifiX: Component<IconComponentProps, {}, "">;
type WifiX = ReturnType<typeof WifiX>;
export default WifiX;
