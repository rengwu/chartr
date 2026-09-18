import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonSimpleSwimIcon` instead.
 *
 * @example
 * ```svelte
 * <PersonSimpleSwim color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PersonSimpleSwim: Component<IconComponentProps, {}, "">;
type PersonSimpleSwim = ReturnType<typeof PersonSimpleSwim>;
export default PersonSimpleSwim;
