import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SolarPanelIcon` instead.
 *
 * @example
 * ```svelte
 * <SolarPanel color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SolarPanel: Component<IconComponentProps, {}, "">;
type SolarPanel = ReturnType<typeof SolarPanel>;
export default SolarPanel;
