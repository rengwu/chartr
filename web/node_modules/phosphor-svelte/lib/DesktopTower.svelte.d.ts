import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DesktopTowerIcon` instead.
 *
 * @example
 * ```svelte
 * <DesktopTower color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DesktopTower: Component<IconComponentProps, {}, "">;
type DesktopTower = ReturnType<typeof DesktopTower>;
export default DesktopTower;
