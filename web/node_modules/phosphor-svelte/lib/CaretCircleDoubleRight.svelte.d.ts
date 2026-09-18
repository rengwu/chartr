import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleDoubleRightIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleDoubleRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleDoubleRight: Component<IconComponentProps, {}, "">;
type CaretCircleDoubleRight = ReturnType<typeof CaretCircleDoubleRight>;
export default CaretCircleDoubleRight;
