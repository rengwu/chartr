import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DiceTwoIcon` instead.
 *
 * @example
 * ```svelte
 * <DiceTwo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DiceTwo: Component<IconComponentProps, {}, "">;
type DiceTwo = ReturnType<typeof DiceTwo>;
export default DiceTwo;
