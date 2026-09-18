import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PixLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <PixLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PixLogo: Component<IconComponentProps, {}, "">;
type PixLogo = ReturnType<typeof PixLogo>;
export default PixLogo;
