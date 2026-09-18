import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NetworkSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <NetworkSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NetworkSlash: Component<IconComponentProps, {}, "">;
type NetworkSlash = ReturnType<typeof NetworkSlash>;
export default NetworkSlash;
