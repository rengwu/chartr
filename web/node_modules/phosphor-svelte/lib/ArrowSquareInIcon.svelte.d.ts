import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowSquareInIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareInIcon: Component<IconComponentProps, {}, "">;
type ArrowSquareInIcon = ReturnType<typeof ArrowSquareInIcon>;
export default ArrowSquareInIcon;
