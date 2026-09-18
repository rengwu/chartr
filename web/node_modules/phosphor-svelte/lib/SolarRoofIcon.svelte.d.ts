import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <SolarRoofIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SolarRoofIcon: Component<IconComponentProps, {}, "">;
type SolarRoofIcon = ReturnType<typeof SolarRoofIcon>;
export default SolarRoofIcon;
