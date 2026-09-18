import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StarFourIcon` instead.
 *
 * @example
 * ```svelte
 * <StarFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StarFour: Component<IconComponentProps, {}, "">;
type StarFour = ReturnType<typeof StarFour>;
export default StarFour;
