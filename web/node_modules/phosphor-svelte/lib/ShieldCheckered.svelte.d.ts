import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldCheckeredIcon` instead.
 *
 * @example
 * ```svelte
 * <ShieldCheckered color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShieldCheckered: Component<IconComponentProps, {}, "">;
type ShieldCheckered = ReturnType<typeof ShieldCheckered>;
export default ShieldCheckered;
