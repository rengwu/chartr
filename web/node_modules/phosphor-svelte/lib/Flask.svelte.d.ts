import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlaskIcon` instead.
 *
 * @example
 * ```svelte
 * <Flask color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Flask: Component<IconComponentProps, {}, "">;
type Flask = ReturnType<typeof Flask>;
export default Flask;
