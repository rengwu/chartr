import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ForkKnifeIcon` instead.
 *
 * @example
 * ```svelte
 * <ForkKnife color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ForkKnife: Component<IconComponentProps, {}, "">;
type ForkKnife = ReturnType<typeof ForkKnife>;
export default ForkKnife;
