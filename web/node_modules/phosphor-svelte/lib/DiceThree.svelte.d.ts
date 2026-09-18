import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DiceThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <DiceThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DiceThree: Component<IconComponentProps, {}, "">;
type DiceThree = ReturnType<typeof DiceThree>;
export default DiceThree;
