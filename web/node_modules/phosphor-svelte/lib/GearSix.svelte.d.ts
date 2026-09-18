import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GearSixIcon` instead.
 *
 * @example
 * ```svelte
 * <GearSix color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GearSix: Component<IconComponentProps, {}, "">;
type GearSix = ReturnType<typeof GearSix>;
export default GearSix;
