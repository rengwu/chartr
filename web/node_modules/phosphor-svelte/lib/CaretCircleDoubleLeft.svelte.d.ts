import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleDoubleLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleDoubleLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleDoubleLeft: Component<IconComponentProps, {}, "">;
type CaretCircleDoubleLeft = ReturnType<typeof CaretCircleDoubleLeft>;
export default CaretCircleDoubleLeft;
