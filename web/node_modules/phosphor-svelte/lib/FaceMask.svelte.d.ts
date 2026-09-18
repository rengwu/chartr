import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FaceMaskIcon` instead.
 *
 * @example
 * ```svelte
 * <FaceMask color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FaceMask: Component<IconComponentProps, {}, "">;
type FaceMask = ReturnType<typeof FaceMask>;
export default FaceMask;
