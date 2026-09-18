import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FingerprintSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <FingerprintSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FingerprintSimple: Component<IconComponentProps, {}, "">;
type FingerprintSimple = ReturnType<typeof FingerprintSimple>;
export default FingerprintSimple;
