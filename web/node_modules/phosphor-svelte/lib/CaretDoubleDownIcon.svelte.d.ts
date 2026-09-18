import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CaretDoubleDownIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CaretDoubleDownIcon: Component<IconComponentProps, {}, "">;
type CaretDoubleDownIcon = ReturnType<typeof CaretDoubleDownIcon>;
export default CaretDoubleDownIcon;
