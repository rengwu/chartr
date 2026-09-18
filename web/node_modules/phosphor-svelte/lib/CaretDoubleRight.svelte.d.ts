import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretDoubleRightIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretDoubleRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretDoubleRight: Component<IconComponentProps, {}, "">;
type CaretDoubleRight = ReturnType<typeof CaretDoubleRight>;
export default CaretDoubleRight;
