import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <FinnTheHumanIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FinnTheHumanIcon: Component<IconComponentProps, {}, "">;
type FinnTheHumanIcon = ReturnType<typeof FinnTheHumanIcon>;
export default FinnTheHumanIcon;
