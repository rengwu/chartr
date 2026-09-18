import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PersonIcon` instead.
 *
 * @example
 * ```svelte
 * <Person color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Person: Component<IconComponentProps, {}, "">;
type Person = ReturnType<typeof Person>;
export default Person;
