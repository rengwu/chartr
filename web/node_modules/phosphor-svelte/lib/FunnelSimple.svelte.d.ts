import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FunnelSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <FunnelSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FunnelSimple: Component<IconComponentProps, {}, "">;
type FunnelSimple = ReturnType<typeof FunnelSimple>;
export default FunnelSimple;
