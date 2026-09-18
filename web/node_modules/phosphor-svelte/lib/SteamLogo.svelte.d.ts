import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SteamLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <SteamLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SteamLogo: Component<IconComponentProps, {}, "">;
type SteamLogo = ReturnType<typeof SteamLogo>;
export default SteamLogo;
