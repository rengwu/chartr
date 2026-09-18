import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleBikeIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleBike color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleBike: Component<IconComponentProps, {}, "">;
type PersonSimpleBike = ReturnType<typeof PersonSimpleBike>;
export default PersonSimpleBike;
