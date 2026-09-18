import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CaretLineLeftIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretLineLeftIcon: Component<IconComponentProps, {}, "">;
type CaretLineLeftIcon = ReturnType<typeof CaretLineLeftIcon>;
export default CaretLineLeftIcon;
