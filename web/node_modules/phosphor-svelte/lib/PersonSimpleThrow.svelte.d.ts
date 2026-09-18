import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleThrowIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleThrow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleThrow: Component<IconComponentProps, {}, "">;
type PersonSimpleThrow = ReturnType<typeof PersonSimpleThrow>;
export default PersonSimpleThrow;
