import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <MedalMilitaryIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MedalMilitaryIcon: Component<IconComponentProps, {}, "">;
type MedalMilitaryIcon = ReturnType<typeof MedalMilitaryIcon>;
export default MedalMilitaryIcon;
