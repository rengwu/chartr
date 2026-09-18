import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DiceOneIcon` instead.
 *
 * @example
 * ```svelte
 * <DiceOne color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DiceOne: Component<IconComponentProps, {}, "">;
type DiceOne = ReturnType<typeof DiceOne>;
export default DiceOne;
