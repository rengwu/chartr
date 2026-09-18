import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <BoxArrowUpIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BoxArrowUpIcon: Component<IconComponentProps, {}, "">;
type BoxArrowUpIcon = ReturnType<typeof BoxArrowUpIcon>;
export default BoxArrowUpIcon;
