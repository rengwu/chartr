import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GuitarIcon` instead.
 *
 * @example
 * ```svelte
 * <Guitar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Guitar: Component<IconComponentProps, {}, "">;
type Guitar = ReturnType<typeof Guitar>;
export default Guitar;
