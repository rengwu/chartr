import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretRightIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretRight: Component<IconComponentProps, {}, "">;
type CaretRight = ReturnType<typeof CaretRight>;
export default CaretRight;
