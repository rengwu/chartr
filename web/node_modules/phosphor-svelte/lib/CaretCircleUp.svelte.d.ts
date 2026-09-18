import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretCircleUpIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretCircleUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleUp: Component<IconComponentProps, {}, "">;
type CaretCircleUp = ReturnType<typeof CaretCircleUp>;
export default CaretCircleUp;
