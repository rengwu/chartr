import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GenderIntersexIcon` instead.
 *
 * @example
 * ```svelte
 * <GenderIntersex color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GenderIntersex: Component<IconComponentProps, {}, "">;
type GenderIntersex = ReturnType<typeof GenderIntersex>;
export default GenderIntersex;
