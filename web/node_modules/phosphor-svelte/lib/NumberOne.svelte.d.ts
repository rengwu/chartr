import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberOneIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberOne color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberOne: Component<IconComponentProps, {}, "">;
type NumberOne = ReturnType<typeof NumberOne>;
export default NumberOne;
