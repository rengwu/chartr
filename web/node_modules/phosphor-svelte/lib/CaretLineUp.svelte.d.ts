import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CaretLineUpIcon` instead.
 *
 * @example
 * ```svelte
 * <CaretLineUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretLineUp: Component<IconComponentProps, {}, "">;
type CaretLineUp = ReturnType<typeof CaretLineUp>;
export default CaretLineUp;
