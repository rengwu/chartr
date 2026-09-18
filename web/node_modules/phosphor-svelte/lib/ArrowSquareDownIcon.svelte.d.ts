import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowSquareDownIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareDownIcon: Component<IconComponentProps, {}, "">;
type ArrowSquareDownIcon = ReturnType<typeof ArrowSquareDownIcon>;
export default ArrowSquareDownIcon;
