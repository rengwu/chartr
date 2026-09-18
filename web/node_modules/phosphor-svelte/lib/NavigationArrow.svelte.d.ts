import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NavigationArrowIcon` instead.
 *
 * @example
 * ```svelte
 * <NavigationArrow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NavigationArrow: Component<IconComponentProps, {}, "">;
type NavigationArrow = ReturnType<typeof NavigationArrow>;
export default NavigationArrow;
