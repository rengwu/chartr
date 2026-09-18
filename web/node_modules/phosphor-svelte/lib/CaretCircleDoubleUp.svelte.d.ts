import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleDoubleUpIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleDoubleUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleDoubleUp: Component<IconComponentProps, {}, "">;
type CaretCircleDoubleUp = ReturnType<typeof CaretCircleDoubleUp>;
export default CaretCircleDoubleUp;
