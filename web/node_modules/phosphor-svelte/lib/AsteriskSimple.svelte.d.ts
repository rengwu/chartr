import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AsteriskSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <AsteriskSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const AsteriskSimple: Component<IconComponentProps, {}, "">;
type AsteriskSimple = ReturnType<typeof AsteriskSimple>;
export default AsteriskSimple;
