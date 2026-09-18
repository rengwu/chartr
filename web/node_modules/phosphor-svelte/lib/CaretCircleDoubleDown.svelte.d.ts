import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleDoubleDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleDoubleDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleDoubleDown: Component<IconComponentProps, {}, "">;
type CaretCircleDoubleDown = ReturnType<typeof CaretCircleDoubleDown>;
export default CaretCircleDoubleDown;
