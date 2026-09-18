import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NetworkXIcon` instead.
 *
 * @example
 * ```svelte
 * <NetworkX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NetworkX: Component<IconComponentProps, {}, "">;
type NetworkX = ReturnType<typeof NetworkX>;
export default NetworkX;
