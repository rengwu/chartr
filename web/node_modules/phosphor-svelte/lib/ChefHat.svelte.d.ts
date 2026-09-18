import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ChefHatIcon` instead.
 *
 * @example
 * ```svelte
 * <ChefHat color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ChefHat: Component<IconComponentProps, {}, "">;
type ChefHat = ReturnType<typeof ChefHat>;
export default ChefHat;
