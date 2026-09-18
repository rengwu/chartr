import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <DiamondsFourIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DiamondsFourIcon: Component<IconComponentProps, {}, "">;
type DiamondsFourIcon = ReturnType<typeof DiamondsFourIcon>;
export default DiamondsFourIcon;
