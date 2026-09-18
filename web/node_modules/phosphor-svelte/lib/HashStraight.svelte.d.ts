import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HashStraightIcon` instead.
 *
 * @example
 * ```svelte
 * <HashStraight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HashStraight: Component<IconComponentProps, {}, "">;
type HashStraight = ReturnType<typeof HashStraight>;
export default HashStraight;
