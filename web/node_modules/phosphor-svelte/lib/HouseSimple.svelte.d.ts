import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HouseSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <HouseSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HouseSimple: Component<IconComponentProps, {}, "">;
type HouseSimple = ReturnType<typeof HouseSimple>;
export default HouseSimple;
