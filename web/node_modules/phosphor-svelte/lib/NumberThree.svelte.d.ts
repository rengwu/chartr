import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberThree: Component<IconComponentProps, {}, "">;
type NumberThree = ReturnType<typeof NumberThree>;
export default NumberThree;
