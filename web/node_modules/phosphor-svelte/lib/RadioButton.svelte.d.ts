import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `RadioButtonIcon` instead.
 *
 * @example
 * ```svelte
 * <RadioButton color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const RadioButton: Component<IconComponentProps, {}, "">;
type RadioButton = ReturnType<typeof RadioButton>;
export default RadioButton;
