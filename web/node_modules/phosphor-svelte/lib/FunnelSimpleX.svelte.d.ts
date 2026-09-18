import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FunnelSimpleXIcon` instead.
 *
 * @example
 * ```svelte
 * <FunnelSimpleX color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FunnelSimpleX: Component<IconComponentProps, {}, "">;
type FunnelSimpleX = ReturnType<typeof FunnelSimpleX>;
export default FunnelSimpleX;
