import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowBendRightDownIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendRightDownIcon: Component<IconComponentProps, {}, "">;
type ArrowBendRightDownIcon = ReturnType<typeof ArrowBendRightDownIcon>;
export default ArrowBendRightDownIcon;
