import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <NetworkSlashIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NetworkSlashIcon: Component<IconComponentProps, {}, "">;
type NetworkSlashIcon = ReturnType<typeof NetworkSlashIcon>;
export default NetworkSlashIcon;
