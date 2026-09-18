import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleCircle: Component<IconComponentProps, {}, "">;
type PersonSimpleCircle = ReturnType<typeof PersonSimpleCircle>;
export default PersonSimpleCircle;
