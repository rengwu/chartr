import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EnvelopeSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <EnvelopeSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EnvelopeSimple: Component<IconComponentProps, {}, "">;
type EnvelopeSimple = ReturnType<typeof EnvelopeSimple>;
export default EnvelopeSimple;
