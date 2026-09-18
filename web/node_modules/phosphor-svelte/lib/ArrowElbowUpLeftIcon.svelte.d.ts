import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ArrowElbowUpLeftIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowUpLeftIcon: Component<IconComponentProps, {}, "">;
type ArrowElbowUpLeftIcon = ReturnType<typeof ArrowElbowUpLeftIcon>;
export default ArrowElbowUpLeftIcon;
