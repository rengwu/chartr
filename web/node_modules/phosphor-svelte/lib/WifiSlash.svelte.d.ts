import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WifiSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <WifiSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WifiSlash: Component<IconComponentProps, {}, "">;
type WifiSlash = ReturnType<typeof WifiSlash>;
export default WifiSlash;
