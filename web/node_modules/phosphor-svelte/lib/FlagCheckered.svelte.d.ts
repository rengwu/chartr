import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlagCheckeredIcon` instead.
 *
 * @example
 * ```svelte
 * <FlagCheckered color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlagCheckered: Component<IconComponentProps, {}, "">;
type FlagCheckered = ReturnType<typeof FlagCheckered>;
export default FlagCheckered;
