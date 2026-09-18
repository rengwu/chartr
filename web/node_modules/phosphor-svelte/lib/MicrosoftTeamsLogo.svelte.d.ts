import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicrosoftTeamsLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MicrosoftTeamsLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftTeamsLogo: Component<IconComponentProps, {}, "">;
type MicrosoftTeamsLogo = ReturnType<typeof MicrosoftTeamsLogo>;
export default MicrosoftTeamsLogo;
