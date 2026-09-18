import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SupersetOfIcon` instead.
 *
 * @example
 * ```svelte
 * <SupersetOf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SupersetOf: Component<IconComponentProps, {}, "">;
type SupersetOf = ReturnType<typeof SupersetOf>;
export default SupersetOf;
