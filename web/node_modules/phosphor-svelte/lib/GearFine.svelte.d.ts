import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GearFineIcon` instead.
 *
 * @example
 * ```svelte
 * <GearFine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GearFine: Component<IconComponentProps, {}, "">;
type GearFine = ReturnType<typeof GearFine>;
export default GearFine;
