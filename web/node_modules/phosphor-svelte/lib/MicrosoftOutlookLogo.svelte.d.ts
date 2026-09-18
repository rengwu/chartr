import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MicrosoftOutlookLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <MicrosoftOutlookLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MicrosoftOutlookLogo: Component<IconComponentProps, {}, "">;
type MicrosoftOutlookLogo = ReturnType<typeof MicrosoftOutlookLogo>;
export default MicrosoftOutlookLogo;
