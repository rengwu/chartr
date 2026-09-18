import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CaretCircleDownIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretCircleDownIcon: Component<IconComponentProps, {}, "">;
type CaretCircleDownIcon = ReturnType<typeof CaretCircleDownIcon>;
export default CaretCircleDownIcon;
