import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <PersonSimpleSkiIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleSkiIcon: Component<IconComponentProps, {}, "">;
type PersonSimpleSkiIcon = ReturnType<typeof PersonSimpleSkiIcon>;
export default PersonSimpleSkiIcon;
