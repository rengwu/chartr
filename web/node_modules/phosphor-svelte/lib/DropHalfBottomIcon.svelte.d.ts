import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <DropHalfBottomIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DropHalfBottomIcon: Component<IconComponentProps, {}, "">;
type DropHalfBottomIcon = ReturnType<typeof DropHalfBottomIcon>;
export default DropHalfBottomIcon;
