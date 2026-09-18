import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretLineLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretLineLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretLineLeft: Component<IconComponentProps, {}, "">;
type CaretLineLeft = ReturnType<typeof CaretLineLeft>;
export default CaretLineLeft;
