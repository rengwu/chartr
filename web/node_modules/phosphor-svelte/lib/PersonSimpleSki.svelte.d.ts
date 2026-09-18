import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleSkiIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleSki color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleSki: Component<IconComponentProps, {}, "">;
type PersonSimpleSki = ReturnType<typeof PersonSimpleSki>;
export default PersonSimpleSki;
