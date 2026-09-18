import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SmileyXEyesIcon` instead.
 *
 * @example
 * ```svelte
 * <SmileyXEyes color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SmileyXEyes: Component<IconComponentProps, {}, "">;
type SmileyXEyes = ReturnType<typeof SmileyXEyes>;
export default SmileyXEyes;
