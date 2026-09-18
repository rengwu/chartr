import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleWalkIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleWalk color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleWalk: Component<IconComponentProps, {}, "">;
type PersonSimpleWalk = ReturnType<typeof PersonSimpleWalk>;
export default PersonSimpleWalk;
