import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DiamondsFourIcon` instead.
 *
 * @example
 * ```svelte
 * <DiamondsFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DiamondsFour: Component<IconComponentProps, {}, "">;
type DiamondsFour = ReturnType<typeof DiamondsFour>;
export default DiamondsFour;
