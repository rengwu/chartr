import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsThreeOutlineVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsThreeOutlineVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsThreeOutlineVertical: Component<IconComponentProps, {}, "">;
type DotsThreeOutlineVertical = ReturnType<typeof DotsThreeOutlineVertical>;
export default DotsThreeOutlineVertical;
