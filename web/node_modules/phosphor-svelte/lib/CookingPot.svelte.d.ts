import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CookingPotIcon` instead.
 *
 * @example
 * ```svelte
 * <CookingPot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CookingPot: Component<IconComponentProps, {}, "">;
type CookingPot = ReturnType<typeof CookingPot>;
export default CookingPot;
