import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleDown: Component<IconComponentProps, {}, "">;
type CaretCircleDown = ReturnType<typeof CaretCircleDown>;
export default CaretCircleDown;
