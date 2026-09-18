import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <RadioactiveIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RadioactiveIcon: Component<IconComponentProps, {}, "">;
type RadioactiveIcon = ReturnType<typeof RadioactiveIcon>;
export default RadioactiveIcon;
