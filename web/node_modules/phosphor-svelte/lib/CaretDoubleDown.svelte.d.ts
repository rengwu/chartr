import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretDoubleDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretDoubleDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretDoubleDown: Component<IconComponentProps, {}, "">;
type CaretDoubleDown = ReturnType<typeof CaretDoubleDown>;
export default CaretDoubleDown;
