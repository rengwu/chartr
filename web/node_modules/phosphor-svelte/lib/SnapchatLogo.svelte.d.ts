import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SnapchatLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SnapchatLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SnapchatLogo: Component<IconComponentProps, {}, "">;
type SnapchatLogo = ReturnType<typeof SnapchatLogo>;
export default SnapchatLogo;
