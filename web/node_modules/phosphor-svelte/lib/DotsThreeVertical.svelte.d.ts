import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsThreeVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsThreeVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsThreeVertical: Component<IconComponentProps, {}, "">;
type DotsThreeVertical = ReturnType<typeof DotsThreeVertical>;
export default DotsThreeVertical;
