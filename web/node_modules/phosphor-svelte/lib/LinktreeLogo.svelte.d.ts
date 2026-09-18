import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LinktreeLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <LinktreeLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LinktreeLogo: Component<IconComponentProps, {}, "">;
type LinktreeLogo = ReturnType<typeof LinktreeLogo>;
export default LinktreeLogo;
