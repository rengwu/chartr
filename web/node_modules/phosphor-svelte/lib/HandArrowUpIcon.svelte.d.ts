import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <HandArrowUpIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandArrowUpIcon: Component<IconComponentProps, {}, "">;
type HandArrowUpIcon = ReturnType<typeof HandArrowUpIcon>;
export default HandArrowUpIcon;
