import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <BabyCarriageIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BabyCarriageIcon: Component<IconComponentProps, {}, "">;
type BabyCarriageIcon = ReturnType<typeof BabyCarriageIcon>;
export default BabyCarriageIcon;
