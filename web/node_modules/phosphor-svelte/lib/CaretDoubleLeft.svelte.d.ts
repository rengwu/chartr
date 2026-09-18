import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretDoubleLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretDoubleLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretDoubleLeft: Component<IconComponentProps, {}, "">;
type CaretDoubleLeft = ReturnType<typeof CaretDoubleLeft>;
export default CaretDoubleLeft;
