import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowBendUpLeftIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendUpLeftIcon: Component<IconComponentProps, {}, "">;
type ArrowBendUpLeftIcon = ReturnType<typeof ArrowBendUpLeftIcon>;
export default ArrowBendUpLeftIcon;
