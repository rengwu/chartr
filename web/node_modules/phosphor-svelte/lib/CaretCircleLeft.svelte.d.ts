import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleLeft: Component<IconComponentProps, {}, "">;
type CaretCircleLeft = ReturnType<typeof CaretCircleLeft>;
export default CaretCircleLeft;
