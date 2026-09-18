import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ComputerTowerIcon` instead.
 *
 * @example
 * ```svelte
 * <ComputerTower color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ComputerTower: Component<IconComponentProps, {}, "">;
type ComputerTower = ReturnType<typeof ComputerTower>;
export default ComputerTower;
