import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CarSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <CarSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CarSimple: Component<IconComponentProps, {}, "">;
type CarSimple = ReturnType<typeof CarSimple>;
export default CarSimple;
