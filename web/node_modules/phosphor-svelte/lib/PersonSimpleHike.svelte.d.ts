import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleHikeIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleHike color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleHike: Component<IconComponentProps, {}, "">;
type PersonSimpleHike = ReturnType<typeof PersonSimpleHike>;
export default PersonSimpleHike;
