import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SignatureIcon` instead.
 *
 * @example
 * ```svelte
 * <Signature color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Signature: Component<IconComponentProps, {}, "">;
type Signature = ReturnType<typeof Signature>;
export default Signature;
