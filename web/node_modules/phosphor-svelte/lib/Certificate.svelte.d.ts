import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CertificateIcon` instead.
 *
 * @example
 * ```svelte
 * <Certificate color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Certificate: Component<IconComponentProps, {}, "">;
type Certificate = ReturnType<typeof Certificate>;
export default Certificate;
