import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <PersonSimpleSwimIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleSwimIcon: Component<IconComponentProps, {}, "">;
type PersonSimpleSwimIcon = ReturnType<typeof PersonSimpleSwimIcon>;
export default PersonSimpleSwimIcon;
