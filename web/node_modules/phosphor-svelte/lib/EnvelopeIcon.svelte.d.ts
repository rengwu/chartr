import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <EnvelopeIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EnvelopeIcon: Component<IconComponentProps, {}, "">;
type EnvelopeIcon = ReturnType<typeof EnvelopeIcon>;
export default EnvelopeIcon;
