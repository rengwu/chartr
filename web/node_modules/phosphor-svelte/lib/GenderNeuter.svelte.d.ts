import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GenderNeuterIcon` instead.
 *
 * @example
 * ```svelte
 * <GenderNeuter color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GenderNeuter: Component<IconComponentProps, {}, "">;
type GenderNeuter = ReturnType<typeof GenderNeuter>;
export default GenderNeuter;
