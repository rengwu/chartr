import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LightningIcon` instead.
 *
 * @example
 * ```svelte
 * <Lightning color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lightning: Component<IconComponentProps, {}, "">;
type Lightning = ReturnType<typeof Lightning>;
export default Lightning;
