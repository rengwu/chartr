import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsThreeCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsThreeCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsThreeCircle: Component<IconComponentProps, {}, "">;
type DotsThreeCircle = ReturnType<typeof DotsThreeCircle>;
export default DotsThreeCircle;
