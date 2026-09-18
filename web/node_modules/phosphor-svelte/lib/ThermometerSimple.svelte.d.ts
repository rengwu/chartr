import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ThermometerSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <ThermometerSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ThermometerSimple: Component<IconComponentProps, {}, "">;
type ThermometerSimple = ReturnType<typeof ThermometerSimple>;
export default ThermometerSimple;
