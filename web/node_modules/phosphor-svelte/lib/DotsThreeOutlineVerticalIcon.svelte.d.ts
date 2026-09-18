import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <DotsThreeOutlineVerticalIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsThreeOutlineVerticalIcon: Component<IconComponentProps, {}, "">;
type DotsThreeOutlineVerticalIcon = ReturnType<typeof DotsThreeOutlineVerticalIcon>;
export default DotsThreeOutlineVerticalIcon;
