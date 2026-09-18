import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CirclesFourIcon` instead.
 *
 * @example
 * ```svelte
 * <CirclesFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CirclesFour: Component<IconComponentProps, {}, "">;
type CirclesFour = ReturnType<typeof CirclesFour>;
export default CirclesFour;
