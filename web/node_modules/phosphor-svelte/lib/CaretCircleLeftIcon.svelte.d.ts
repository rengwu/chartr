import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CaretCircleLeftIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleLeftIcon: Component<IconComponentProps, {}, "">;
type CaretCircleLeftIcon = ReturnType<typeof CaretCircleLeftIcon>;
export default CaretCircleLeftIcon;
