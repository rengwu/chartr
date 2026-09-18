import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsThreeCircleVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsThreeCircleVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsThreeCircleVertical: Component<IconComponentProps, {}, "">;
type DotsThreeCircleVertical = ReturnType<typeof DotsThreeCircleVertical>;
export default DotsThreeCircleVertical;
