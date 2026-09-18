import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <RadioButtonIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RadioButtonIcon: Component<IconComponentProps, {}, "">;
type RadioButtonIcon = ReturnType<typeof RadioButtonIcon>;
export default RadioButtonIcon;
