import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlantIcon` instead.
 *
 * @example
 * ```svelte
 * <Plant color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Plant: Component<IconComponentProps, {}, "">;
type Plant = ReturnType<typeof Plant>;
export default Plant;
