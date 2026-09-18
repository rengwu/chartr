import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EnvelopeOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <EnvelopeOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EnvelopeOpen: Component<IconComponentProps, {}, "">;
type EnvelopeOpen = ReturnType<typeof EnvelopeOpen>;
export default EnvelopeOpen;
