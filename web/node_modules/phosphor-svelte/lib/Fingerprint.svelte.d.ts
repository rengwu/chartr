import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FingerprintIcon` instead.
 *
 * @example
 * ```svelte
 * <Fingerprint color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Fingerprint: Component<IconComponentProps, {}, "">;
type Fingerprint = ReturnType<typeof Fingerprint>;
export default Fingerprint;
