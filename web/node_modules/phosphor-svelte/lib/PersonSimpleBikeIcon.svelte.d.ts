import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <PersonSimpleBikeIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleBikeIcon: Component<IconComponentProps, {}, "">;
type PersonSimpleBikeIcon = ReturnType<typeof PersonSimpleBikeIcon>;
export default PersonSimpleBikeIcon;
