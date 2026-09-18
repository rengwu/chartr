import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretLineDownIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretLineDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretLineDown: Component<IconComponentProps, {}, "">;
type CaretLineDown = ReturnType<typeof CaretLineDown>;
export default CaretLineDown;
