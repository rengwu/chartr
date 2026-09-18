import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BehanceLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <BehanceLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BehanceLogo: Component<IconComponentProps, {}, "">;
type BehanceLogo = ReturnType<typeof BehanceLogo>;
export default BehanceLogo;
