import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretDown: Component<IconComponentProps, {}, "">;
type CaretDown = ReturnType<typeof CaretDown>;
export default CaretDown;
