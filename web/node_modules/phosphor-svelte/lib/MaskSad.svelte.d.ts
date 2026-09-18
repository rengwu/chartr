import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MaskSadIcon` instead.
 *
 * @example
 * ```svelte
 * <MaskSad color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MaskSad: Component<IconComponentProps, {}, "">;
type MaskSad = ReturnType<typeof MaskSad>;
export default MaskSad;
