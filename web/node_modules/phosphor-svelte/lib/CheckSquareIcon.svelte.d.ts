import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <CheckSquareIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CheckSquareIcon: Component<IconComponentProps, {}, "">;
type CheckSquareIcon = ReturnType<typeof CheckSquareIcon>;
export default CheckSquareIcon;
