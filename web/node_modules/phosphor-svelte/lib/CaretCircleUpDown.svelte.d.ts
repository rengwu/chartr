import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleUpDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleUpDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleUpDown: Component<IconComponentProps, {}, "">;
type CaretCircleUpDown = ReturnType<typeof CaretCircleUpDown>;
export default CaretCircleUpDown;
