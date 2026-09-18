import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleSnowboardIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleSnowboard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleSnowboard: Component<IconComponentProps, {}, "">;
type PersonSimpleSnowboard = ReturnType<typeof PersonSimpleSnowboard>;
export default PersonSimpleSnowboard;
