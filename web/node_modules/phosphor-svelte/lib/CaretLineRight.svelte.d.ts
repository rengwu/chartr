import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretLineRightIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretLineRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretLineRight: Component<IconComponentProps, {}, "">;
type CaretLineRight = ReturnType<typeof CaretLineRight>;
export default CaretLineRight;
